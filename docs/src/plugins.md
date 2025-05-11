# Plugis

## `FileExtPlugin`

A `FileExtPlugin` takes a `PathBuf` as parameter and returns a `RuntimeNode`. It also defines its target file extensions.

```rust
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
                            #[cfg(target_family = "unix")]
                            let library = libloading::os::unix::Library::open(
                                Some(path_buf.clone()),
                                libloading::os::unix::RTLD_NOW | libloading::os::unix::RTLD_GLOBAL,
                            )
                            .wrap_err(format!("Failed to load path {:?}", path_buf))?;

                            #[cfg(not(target_family = "unix"))]
                            let library = Library::new(path_buf.clone())
                                .wrap_err(format!("Failed to load path {:?}", path_buf))?;

                            library
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
```

## `UrlSchemePlugin`

An `UrlSchemePlugin` takes a `Url` as parameter and returns a `RuntimeNode`. It also defines its target URL schemes. It can uses a `FileExtPlugin` to load the file extension.

```rust
#[derive(UrlSchemePlugin)]
pub struct DefaultUrlSchemePlugin {}

#[url_scheme_plugin(runtime = "default_runtime")]
impl UrlSchemePlugin for DefaultUrlSchemePlugin {
    async fn new() -> Result<Self>
    where
        Self: Sized,
    {
        Ok(DefaultUrlSchemePlugin {})
    }

    fn target(&self) -> Vec<String> {
        vec!["file".to_string(), "builtin".to_string()]
    }

    #[allow(clippy::too_many_arguments)]
    async fn load(
        &self,
        url: Url,

        inputs: Inputs,
        outputs: Outputs,
        queries: Queries,
        queryables: Queryables,
        configuration: serde_yml::Value,
        file_ext: Arc<FileExtManager>,
    ) -> Result<iridis_runtime_core::prelude::RuntimeNode> {
        match url.scheme() {
            "file" => {
                let path = url
                    .to_file_path()
                    .map_err(|_| eyre::eyre!("Url '{}' cannot be made into a path buf", url))?;

                file_ext
                    .load(path, inputs, outputs, queries, queryables, configuration)
                    .await
            }
            "builtin" => Ok(RuntimeNode::StaticallyLinked(
                new_builtin(
                    Builtin::from_string(url.path())
                        .wrap_err(format!("Invalid builtin name '{}'", url.path()))?,
                    inputs,
                    outputs,
                    queries,
                    queryables,
                    configuration,
                )
                .await?,
            )),
            _ => Err(eyre::eyre!(
                "Url scheme '{}' is not supported",
                url.scheme()
            )),
        }
    }
}
```
