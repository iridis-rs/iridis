use libloading::Library;

use crate::prelude::*;

pub struct DynamicallyLinkedNode {
    pub handle: Box<dyn Node>,
    pub _library: Library,
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
        }
    }
}
