//! This module contains the `UrlSchemePlugin` trait and its implementations.

use std::sync::Arc;

use crate::prelude::{thirdparty::tokio::task::JoinHandle, *};

/// This trait must be implemented in order to make a plugin compatible with the `iridis_url_scheme` crate.
pub trait UrlSchemePlugin: Send + Sync {
    /// Thisfunction is called when the plugin is loaded.
    #[allow(clippy::new_ret_no_self)]
    fn new() -> JoinHandle<Result<Box<dyn UrlSchemePlugin>>>
    where
        Self: Sized;

    /// This function is called when the plugin is loaded to determine which URL schemes it supports.
    fn target(&self) -> Vec<String>;

    /// This function is called to load a `Node` from the plugin.
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
    ) -> JoinHandle<Result<RuntimeNode>>;
}

/// This type is used to represent the return type of the `C` symbolic function that instantiates the plugin.
pub type DynamicallyLinkedUrlSchemePluginInstance =
    fn() -> JoinHandle<Result<Box<dyn UrlSchemePlugin>>>;
