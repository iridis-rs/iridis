use std::{collections::HashMap, sync::Arc};

use crate::prelude::*;

/// Loader struct passed to the user closure to load nodes
pub struct NodeLoader {
    pub file_ext: Arc<FileExtManager>,
    pub url_scheme: Arc<UrlSchemeManager>,

    pub clock: Arc<HLC>,

    pub flows: Flows,
    pub nodes: HashMap<NodeLayout, RuntimeNode>,
}

impl NodeLoader {
    pub fn new(
        file_ext: Arc<FileExtManager>,
        url_scheme: Arc<UrlSchemeManager>,
        clock: Arc<HLC>,
        flows: Flows,
    ) -> Self {
        Self {
            file_ext,
            url_scheme,
            clock,
            flows,
            nodes: HashMap::new(),
        }
    }

    /// Load a node from a Rust struct directly (statically linked)
    pub async fn load<T: Node + 'static>(
        &mut self,
        source: NodeLayout,
        configuration: serde_yml::Value,
    ) -> Result<()> {
        let (inputs, outputs, queries, queryables) =
            self.flows.node_io(self.clock.clone(), source.clone());

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

        self.nodes.insert(source, node);
        Ok(())
    }

    /// Load a node from an URL. Be careful, you must ensure that the runtime has the necessary plugins to process this URL.
    /// By default you can pass all URL for the builtins nodes (builtin://) and all URL for dynamic libraries on the computer (file:///path/to/library.so)
    pub async fn load_url(
        &mut self,
        url: Url,
        source: NodeLayout,
        configuration: serde_yml::Value,
    ) -> Result<()> {
        let (inputs, outputs, queries, queryables) =
            self.flows.node_io(self.clock.clone(), source.clone());

        let node = self
            .url_scheme
            .load(
                url.clone(),
                inputs,
                outputs,
                queries,
                queryables,
                configuration,
                self.file_ext.clone(),
            )
            .await?;

        tracing::debug!(
            "Node '{}' (uuid: {}) loaded from URL {:?}",
            source.label,
            source.uuid,
            url
        );

        self.nodes.insert(source, node);

        Ok(())
    }
}
