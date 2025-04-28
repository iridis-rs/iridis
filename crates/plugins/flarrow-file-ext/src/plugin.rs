use std::path::PathBuf;

use crate::prelude::{thirdparty::tokio::task::JoinHandle, *};

pub trait FileExtPlugin: Send + Sync {
    #[allow(clippy::new_ret_no_self)]
    fn new() -> JoinHandle<Result<Box<dyn FileExtPlugin>>>
    where
        Self: Sized;

    fn target(&self) -> Vec<String>;

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

pub type DynamicallyLinkedFileExtPluginInstance =
    fn() -> JoinHandle<Result<Box<dyn FileExtPlugin>>>;
