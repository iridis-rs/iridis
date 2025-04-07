use std::{collections::HashMap, sync::Arc};

use libloading::Library;
use url::Url;

use crate::prelude::*;

pub struct DynamicallyLinkedNode {
    handle: Box<dyn Node>,
    _library: Library,
}

pub enum RuntimeNode {
    StaticallyLinked(Box<dyn Node>),
    DynamicallyLinked(DynamicallyLinkedNode),
}

impl RuntimeNode {
    pub async fn run(self) -> Result<()> {
        match self {
            RuntimeNode::StaticallyLinked(node) => node
                .start()
                .await
                .wrap_err("Failed to await statically linked node")?,
            RuntimeNode::DynamicallyLinked(node) => node
                .handle
                .start()
                .await
                .wrap_err("Failed to await dynamically linked node")?,
        } // TODO: make sure library is not dropped until the node has finished running (after awaiting it)?
    }
}

pub struct Loader {
    pub flows: Flows,

    pub clock: Arc<uhlc::HLC>,
    pub nodes: HashMap<NodeID, RuntimeNode>,
}

impl Loader {
    pub fn new(flows: Flows, clock: Arc<uhlc::HLC>) -> Self {
        Loader {
            flows,
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

        let kind = flarrow_url::prelude::process_url(url.clone())
            .await
            .wrap_err(eyre::eyre!("Could not process url {:?}", url))?;

        match kind {
            flarrow_url::prelude::NodeKind::Builtin(builtin) => {
                self.nodes.insert(
                    node,
                    RuntimeNode::StaticallyLinked(
                        flarrow_builtins::prelude::new_builtin(
                            builtin,
                            inputs,
                            outputs,
                            configuration,
                        )
                        .await
                        .wrap_err("Failed to create statically linked node")?,
                    ),
                );
            }
            flarrow_url::prelude::NodeKind::DynamicallyLinkedLibrary(path) => {
                let library = unsafe { Library::new(path)? };
                let constructor = unsafe {
                    library
                        .get::<*mut DynamicallyLinkedNodeInstance>(b"FLARROW_NODE")?
                        .read()
                };

                let inputs = Inputs::new(node, self.flows.receivers.clone());
                let outputs = Outputs::new(node, self.clock.clone(), self.flows.senders.clone());

                self.nodes.insert(
                    node,
                    RuntimeNode::DynamicallyLinked(DynamicallyLinkedNode {
                        _library: library,
                        handle: (constructor)(inputs, outputs, configuration)
                            .await
                            .wrap_err("Failed to await for dynamically linked node")?
                            .wrap_err("Failed to create dynamically linked node")?,
                    }),
                );
            }
            flarrow_url::prelude::NodeKind::PythonScript(_) => unimplemented!(),
        }

        Ok(())
    }
}
