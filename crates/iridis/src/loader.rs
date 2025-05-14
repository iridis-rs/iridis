use std::{collections::HashMap, sync::Arc};

use crate::prelude::{thirdparty::tokio::task::JoinSet, *};

/// Loader struct passed to the user closure to load nodes
pub struct Loader {
    pub file_ext: Arc<FileExtManager>,
    pub url_scheme: Arc<UrlSchemeManager>,

    pub clock: Arc<HLC>,

    pub flows: RuntimeFlows,

    pub futures: JoinSet<Result<(NodeID, RuntimeNode)>>,
}

impl Loader {
    pub fn new(
        file_ext: Arc<FileExtManager>,
        url_scheme: Arc<UrlSchemeManager>,
        clock: Arc<HLC>,
        flows: RuntimeFlows,
    ) -> Self {
        Self {
            file_ext,
            url_scheme,
            clock,
            flows,
            futures: JoinSet::new(),
        }
    }

    /// Load a node from a Rust struct directly (statically linked)
    pub fn load<T: Node + 'static>(&mut self, source: NodeID, configuration: serde_yml::Value) {
        let (inputs, outputs, queries, queryables) = self
            .flows
            .node_primitives(self.clock.clone(), source.clone());

        self.futures.spawn(async move {
            let node = RuntimeNode::StaticallyLinked(
                T::new(inputs, outputs, queries, queryables, configuration)
                    .await?
                    .wrap_err(format!(
                        "Node '{}' (uuid: {}) failed to initialize",
                        source.label, source.uuid,
                    ))?,
            );

            tracing::debug!(
                "Node '{}' (uuid: {}) loaded from Rust struct {}",
                source.label,
                source.uuid,
                std::any::type_name::<T>()
            );

            Ok((source, node))
        });
    }

    /// Load a node from an URL. Be careful, you must ensure that the runtime has the necessary plugins to process this URL.
    /// By default you can pass all URL for the builtins nodes (builtin://) and all URL for dynamic libraries on the computer (file:///path/to/library.so)
    pub fn load_url(&mut self, url: Url, source: NodeID, configuration: serde_yml::Value) {
        let (inputs, outputs, queries, queryables) = self
            .flows
            .node_primitives(self.clock.clone(), source.clone());

        let file_ext = self.file_ext.clone();
        let url_scheme = self.url_scheme.clone();

        self.futures.spawn(async move {
            let node = url_scheme
                .load(
                    url.clone(),
                    inputs,
                    outputs,
                    queries,
                    queryables,
                    configuration,
                    file_ext,
                )
                .await?;

            tracing::debug!(
                "Node '{}' (uuid: {}) loaded from URL {:?}",
                source.label,
                source.uuid,
                url
            );

            Ok((source, node))
        });
    }

    pub async fn finish(mut self) -> Result<HashMap<NodeID, RuntimeNode>> {
        let mut nodes = HashMap::new();

        while let Some(res) = self.futures.join_next().await {
            let (source, node) = res??;

            nodes.insert(source, node);
        }

        Ok(nodes)
    }
}
