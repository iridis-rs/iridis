//! This module defines the default url scheme plugin for the `iridis` runtime.

use std::sync::Arc;

use crate::prelude::*;

/// The actual struct that implements the `UrlSchemePlugin` trait. This Default
/// plugin loads files and builtin nodes.
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

    /// The url schemes that this plugin supports, by default it is `file` and
    /// `builtin`
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
