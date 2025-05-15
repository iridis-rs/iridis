//! This module defines the `RuntimeNode` enum, which can represent either a statically linked or dynamically linked node.
//! It's separated from other modules because it's fundamentally a brick of the runtime.

use crate::prelude::{thirdparty::libloading, *};

/// This struct represents a dynamically linked node.
/// It loads the node from a shared library at runtime, storing the handle as a `Box<dyn Node>`.
/// It's really important to store the library as well, because once the library is dropped the handle will be invalid.
pub struct DynamicallyLinkedNode {
    /// The `Node` object the runtime will use
    pub handle: Box<dyn Node>,

    #[cfg(not(target_family = "unix"))]
    pub _library: libloading::Library,
    #[cfg(target_family = "unix")]
    pub _library: libloading::os::unix::Library,
}

/// This is the main enum of this module. It represents a node that can be either statically linked or dynamically linked,
/// allowing the runtime to use either type of node interchangeably.
pub enum RuntimeNode {
    /// A statically linked node, which is a concrete implementation of the `Node` trait.
    StaticallyLinked(Box<dyn Node>),

    /// A dynamically linked node, which is loaded from a shared library at runtime.
    DynamicallyLinked(DynamicallyLinkedNode),
}

impl RuntimeNode {
    /// This function will either start the statically linked node or the dynamically linked node,
    /// awaiting for the result.
    pub async fn run(self) -> Result<()> {
        match self {
            RuntimeNode::StaticallyLinked(node) => node.start().await?,
            RuntimeNode::DynamicallyLinked(node) => node.handle.start().await?,
        }
    }
}
