use crate::prelude::{thirdparty::libloading, *};

pub struct DynamicallyLinkedNode {
    pub handle: Box<dyn Node>,

    #[cfg(not(target_family = "unix"))]
    pub _library: libloading::Library,
    #[cfg(target_family = "unix")]
    pub _library: libloading::os::unix::Library,
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
