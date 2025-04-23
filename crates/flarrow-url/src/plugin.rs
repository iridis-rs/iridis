use std::path::PathBuf;

use libloading::Library;
use tokio::task::JoinHandle;
use url::Url;

use crate::prelude::*;

pub trait UrlPlugin: Send {
    #[allow(clippy::new_ret_no_self)]
    fn new() -> JoinHandle<Result<Box<dyn UrlPlugin>>>
    where
        Self: Sized;

    fn load(
        &self,
        url: Url,
        inputs: Inputs,
        outputs: Outputs,
        queries: Queries,
        queryables: Queryables,
        configuration: serde_yml::Value,
    ) -> JoinHandle<Result<RuntimeNode>>;
}

pub type DynamicallyLinkedUrlPluginInstance = fn() -> JoinHandle<Result<Box<dyn UrlPlugin>>>;

pub struct DynamicallyLinkedUrlPlugin {
    handle: Box<dyn UrlPlugin>, // The order is really important, the _library must be dropped after the handle
    _library: Library,
}

pub enum RuntimeUrlPlugin {
    StaticallyLinked(Box<dyn UrlPlugin>),
    DynamicallyLinked(DynamicallyLinkedUrlPlugin),
}

impl RuntimeUrlPlugin {
    pub async fn new_statically_linked<T: UrlPlugin + 'static>() -> Result<Self> {
        Ok(Self::StaticallyLinked(
            T::new()
                .await
                .wrap_err("Failed to await for statically linked url plugin")?
                .wrap_err("Failed to create statically linked url plugin")?,
        ))
    }

    pub async fn new_dynamically_linked(path: PathBuf) -> Result<Self> {
        let library = unsafe { Library::new(path)? };
        let constructor = unsafe {
            library
                .get::<*mut DynamicallyLinkedUrlPluginInstance>(b"FLARROW_URL_PLUGIN")?
                .read()
        };

        Ok(RuntimeUrlPlugin::DynamicallyLinked(
            DynamicallyLinkedUrlPlugin {
                _library: library,
                handle: (constructor)()
                    .await
                    .wrap_err("Failed to await for dynamically linked url plugin")?
                    .wrap_err("Failed to create dynamically linked url plugin")?,
            },
        ))
    }

    pub fn load(
        &self,
        url: Url,
        inputs: Inputs,
        outputs: Outputs,
        queries: Queries,
        queryables: Queryables,
        configuration: serde_yml::Value,
    ) -> JoinHandle<Result<RuntimeNode>> {
        match self {
            RuntimeUrlPlugin::StaticallyLinked(plugin) => {
                plugin.load(url, inputs, outputs, queries, queryables, configuration)
            }
            RuntimeUrlPlugin::DynamicallyLinked(plugin) => {
                plugin
                    .handle
                    .load(url, inputs, outputs, queries, queryables, configuration)
            }
        }
    }
}
