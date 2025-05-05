use crate::prelude::{thirdparty::libloading::Library, *};

#[derive(FileExtPlugin)]
pub struct DefaultFileExtPlugin {}

#[file_ext_plugin(runtime = "default_runtime")]
impl FileExtPlugin for DefaultFileExtPlugin {
    async fn new() -> Result<Self>
    where
        Self: Sized,
    {
        Ok(DefaultFileExtPlugin {})
    }

    fn target(&self) -> Vec<String> {
        vec!["so".to_string(), "dylib".to_string(), "dll".to_string()]
    }

    async fn load(
        &self,
        path: std::path::PathBuf,

        inputs: Inputs,
        outputs: Outputs,
        queries: Queries,
        queryables: Queryables,
        configuration: serde_yml::Value,
    ) -> Result<iridis_runtime_core::prelude::RuntimeNode> {
        match path.extension() {
            Some(ext) => {
                if ext == std::env::consts::DLL_EXTENSION {
                    let path_buf = path.clone();
                    let (library, constructor) = tokio::task::spawn_blocking(move || {
                        let library = unsafe {
                            Library::new(path_buf.clone())
                                .wrap_err(format!("Failed to load path {:?}", path_buf))?
                        };

                        let constructor = unsafe {
                            library
                                .get::<*mut DynamicallyLinkedNodeInstance>(b"IRIDIS_NODE")
                                .wrap_err(format!(
                                    "Failed to load symbol 'IRIDIS_NODE' from dylib {:?}",
                                    path_buf
                                ))?
                                .read()
                        };

                        Ok::<_, eyre::Report>((library, constructor))
                    })
                    .await??;

                    Ok(RuntimeNode::DynamicallyLinked(DynamicallyLinkedNode {
                        _library: library,
                        handle: (constructor)(inputs, outputs, queries, queryables, configuration)
                            .await?
                            .wrap_err(format!(
                                "Failed to create dynamically linked node from dylib {:?}",
                                path,
                            ))?,
                    }))
                } else {
                    Err(eyre::eyre!(
                        "Unsupported file extension '{:?}'. On this platform it must be '{}'",
                        ext,
                        std::env::consts::DLL_EXTENSION
                    ))
                }
            }
            None => Err(eyre::eyre!("No file extension found for path {:?}", path)),
        }
    }
}
