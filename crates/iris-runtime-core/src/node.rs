use crate::prelude::{thirdparty::libloading::Library, *};

pub struct DynamicallyLinkedNode {
    pub handle: Box<dyn Node>,
    pub _library: Library,
}

pub enum RuntimeNode {
    StaticallyLinked(Box<dyn Node>),
    DynamicallyLinked(DynamicallyLinkedNode),
}

impl RuntimeNode {
    /// This function will async wait the node termination (either success or error)
    pub async fn run(self) -> Result<()> {
        match self {
            RuntimeNode::StaticallyLinked(node) => node.start().await?,
            RuntimeNode::DynamicallyLinked(node) => node.handle.start().await?,
        }
    }
}
