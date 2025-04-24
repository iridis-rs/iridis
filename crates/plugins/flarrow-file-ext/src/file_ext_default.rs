use crate::prelude::{thirdparty::libloading::Library, *};

#[doc(hidden)]
#[unsafe(no_mangle)]
pub static FLARROW_FILE_EXT_PLUGIN: DynamicallyLinkedFileExtPluginInstance =
    || DefaultFileExtPlugin::new();

static DEFAULT_TOKIO_RUNTIME: std::sync::LazyLock<tokio::runtime::Runtime> =
    std::sync::LazyLock::new(|| {
        tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime")
    });

fn default_runtime<T: Send + 'static>(
    task: impl Future<Output = T> + Send + 'static,
) -> tokio::task::JoinHandle<T> {
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => handle.spawn(task),
        Err(_) => DEFAULT_TOKIO_RUNTIME.spawn(task),
    }
}

pub struct DefaultFileExtPlugin {}

impl FileExtPlugin for DefaultFileExtPlugin {
    fn new() -> tokio::task::JoinHandle<Result<Box<dyn FileExtPlugin>>>
    where
        Self: Sized,
    {
        default_runtime(
            async move { Ok(Box::new(DefaultFileExtPlugin {}) as Box<dyn FileExtPlugin>) },
        )
    }

    fn target(&self) -> Vec<String> {
        vec!["so".to_string(), "dylib".to_string(), "dll".to_string()]
    }

    fn load(
        &self,
        path: std::path::PathBuf,

        inputs: Inputs,
        outputs: Outputs,
        queries: Queries,
        queryables: Queryables,
        configuration: serde_yml::Value,
    ) -> tokio::task::JoinHandle<Result<flarrow_runtime_core::prelude::RuntimeNode>> {
        default_runtime(async move {
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
                                    .get::<*mut DynamicallyLinkedNodeInstance>(b"FLARROW_NODE")
                                    .wrap_err(format!(
                                        "Failed to load symbol 'FLARROW_NODE' from dylib {:?}",
                                        path_buf
                                    ))?
                                    .read()
                            };

                            Ok::<_, eyre::Report>((library, constructor))
                        })
                        .await??;

                        Ok(RuntimeNode::DynamicallyLinked(DynamicallyLinkedNode {
                            _library: library,
                            handle: (constructor)(
                                inputs,
                                outputs,
                                queries,
                                queryables,
                                configuration,
                            )
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
        })
    }
}
