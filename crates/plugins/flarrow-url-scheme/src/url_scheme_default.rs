use std::sync::Arc;

use crate::prelude::*;

#[doc(hidden)]
#[unsafe(no_mangle)]
pub static FLARROW_URL_SCHEME_PLUGIN: DynamicallyLinkedUrlSchemePluginInstance =
    || DefaultUrlSchemePlugin::new();

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

pub struct DefaultUrlSchemePlugin {}

impl UrlSchemePlugin for DefaultUrlSchemePlugin {
    fn new() -> tokio::task::JoinHandle<Result<Box<dyn UrlSchemePlugin>>>
    where
        Self: Sized,
    {
        default_runtime(async move {
            Ok(Box::new(DefaultUrlSchemePlugin {}) as Box<dyn UrlSchemePlugin>)
        })
    }

    fn target(&self) -> Vec<String> {
        vec!["file".to_string(), "builtin".to_string()]
    }

    #[allow(clippy::too_many_arguments)]
    fn load(
        &self,
        url: Url,

        inputs: Inputs,
        outputs: Outputs,
        queries: Queries,
        queryables: Queryables,
        configuration: serde_yml::Value,
        file_ext: Arc<FileExtManager>,
    ) -> tokio::task::JoinHandle<Result<flarrow_runtime_core::prelude::RuntimeNode>> {
        default_runtime(async move {
            match url.scheme() {
                "file" => {
                    let path = url
                        .to_file_path()
                        .map_err(|_| eyre::eyre!("Url '{}' cannot be made into a path buf", url))?;

                    file_ext
                        .load(path, inputs, outputs, queries, queryables, configuration)
                        .await
                }
                "builtin" => {
                    // TODO!!
                    eyre::bail!("Builtin url scheme is not supported yet!")
                }
                _ => Err(eyre::eyre!(
                    "Url scheme '{}' is not supported",
                    url.scheme()
                )),
            }
        })
    }
}
