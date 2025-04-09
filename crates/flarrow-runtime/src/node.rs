use std::{collections::HashMap, sync::Arc};

use url::Url;

use crate::prelude::*;

pub struct Loader {
    pub flows: Flows,
    pub url_plugin: RuntimeUrlPlugin,

    pub clock: Arc<uhlc::HLC>,
    pub nodes: HashMap<NodeID, RuntimeNode>,
}

impl Loader {
    pub fn new(flows: Flows, url_plugin: RuntimeUrlPlugin, clock: Arc<uhlc::HLC>) -> Self {
        Loader {
            flows,
            url_plugin,
            clock,
            nodes: HashMap::new(),
        }
    }

    pub async fn load_statically_linked<T: Node + 'static>(
        &mut self,
        node: NodeID,
        configuration: serde_yml::Value,
    ) -> eyre::Result<()> {
        let inputs = Inputs::new(node, self.flows.receivers.clone());
        let outputs = Outputs::new(node, self.clock.clone(), self.flows.senders.clone());

        self.nodes.insert(
            node,
            RuntimeNode::StaticallyLinked(
                T::new(inputs, outputs, configuration)
                    .await
                    .wrap_err("Failed to await statically linked node")?
                    .wrap_err("Failed to create statically linked node")?,
            ),
        );

        Ok(())
    }

    pub async fn load_from_url(
        &mut self,
        node: NodeID,
        url: Url,
        configuration: serde_yml::Value,
    ) -> eyre::Result<()> {
        let inputs = Inputs::new(node, self.flows.receivers.clone());
        let outputs = Outputs::new(node, self.clock.clone(), self.flows.senders.clone());

        let handle = self
            .url_plugin
            .load(url.clone(), inputs, outputs, configuration)
            .await
            .wrap_err(format!("Failed to await node from URL: {}", url))?
            .wrap_err(format!("Failed to create node from URL: {}", url))?;

        self.nodes.insert(node, handle);

        Ok(())
    }
}
