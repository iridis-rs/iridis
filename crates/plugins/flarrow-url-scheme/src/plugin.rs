use std::sync::Arc;

use crate::prelude::{thirdparty::tokio::task::JoinHandle, *};

pub trait UrlSchemePlugin: Send + Sync {
    #[allow(clippy::new_ret_no_self)]
    fn new() -> JoinHandle<Result<Box<dyn UrlSchemePlugin>>>
    where
        Self: Sized;

    fn target(&self) -> Vec<String>;

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

pub type DynamicallyLinkedUrlSchemePluginInstance =
    fn() -> JoinHandle<Result<Box<dyn UrlSchemePlugin>>>;
