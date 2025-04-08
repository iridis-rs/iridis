use std::path::PathBuf;

use flarrow_builtins::prelude::{Builtin, new_builtin};
use flarrow_url::prelude::*;
use libloading::Library;

#[doc(hidden)]
#[unsafe(no_mangle)]
pub static FLARROW_URL_PLUGIN: DynamicallyLinkedUrlPluginInstance = || UrlDefaultPlugin::new();

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

pub struct UrlDefaultPlugin {}

impl UrlPlugin for UrlDefaultPlugin {
    fn new() -> JoinHandle<Result<Box<dyn UrlPlugin>>>
    where
        Self: Sized,
    {
        default_runtime(async move { Ok(Box::new(UrlDefaultPlugin {}) as Box<dyn UrlPlugin>) })
    }

    fn load(
        &self,
        url: url::Url,
        inputs: Inputs,
        outputs: Outputs,
        configuration: serde_yml::Value,
    ) -> JoinHandle<Result<RuntimeNode>> {
        default_runtime(async move {
            match url.scheme() {
                "builtin" => {
                    let builtin_name = url.path().trim_start_matches('/').to_string();

                    let builtin = Builtin::from_string(builtin_name)
                        .wrap_err("Failed to parse builtin name")?;

                    Ok(RuntimeNode::StaticallyLinked(
                        new_builtin(builtin, inputs, outputs, configuration)
                            .await
                            .wrap_err("Failed to create builtin node")?,
                    ))
                }
                "file" => {
                    let path = PathBuf::from(url.path());

                    match path.extension() {
                        Some(ext) => {
                            if ext == std::env::consts::DLL_EXTENSION {
                                let library = unsafe { Library::new(path)? };
                                let constructor = unsafe {
                                    library
                                        .get::<*mut DynamicallyLinkedNodeInstance>(b"FLARROW_NODE")?
                                        .read()
                                };

                                Ok(RuntimeNode::DynamicallyLinked(DynamicallyLinkedNode {
                                    _library: library,
                                    handle: (constructor)(inputs, outputs, configuration)
                                        .await
                                        .wrap_err("Failed to await for dynamically linked node")?
                                        .wrap_err("Failed to create dynamically linked node")?,
                                }))
                            } else {
                                Err(eyre::eyre!("Unsupported file extension!"))
                            }
                        }
                        None => Err(eyre::eyre!("No file extension found")),
                    }
                }
                _ => Err(eyre::eyre!("Unsupported scheme")),
            }
        })
    }
}
