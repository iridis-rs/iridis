//! This module contains the `FileExtPlugin` trait and its implementations.

use std::path::PathBuf;

use crate::prelude::{thirdparty::tokio::task::JoinHandle, *};

/// This trait must be implemented in order to make a plugin compatible with the `iridis_file_ext` crate.
pub trait FileExtPlugin: Send + Sync {
    /// This function is called when the plugin is loaded.
    #[allow(clippy::new_ret_no_self)]
    fn new() -> JoinHandle<Result<Box<dyn FileExtPlugin>>>
    where
        Self: Sized;

    /// This function is called when the plugin is loaded to determine which file extensions it supports.
    fn target(&self) -> Vec<String>;

    /// This function is called to load a `Node` from the plugin.
    fn load(
        &self,
        path: PathBuf,

        inputs: Inputs,
        outputs: Outputs,
        queries: Queries,
        queryables: Queryables,
        configuration: serde_yml::Value,
    ) -> JoinHandle<Result<RuntimeNode>>;
}

/// This type is used to represent the return type of the `C` symbolic function that instantiates the plugin.
pub type DynamicallyLinkedFileExtPluginInstance =
    fn() -> JoinHandle<Result<Box<dyn FileExtPlugin>>>;
