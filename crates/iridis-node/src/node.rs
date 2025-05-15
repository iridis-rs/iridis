//! This module defines the `Node` trait that must be implemented
//! by all nodes in the `iridis` runtime.

use crate::prelude::*;

/// The `Node` trait defines the interface for all nodes in the `iridis` runtime.
pub trait Node: Send + Sync {
    /// The `new` function is used to create a new instance of the node.
    #[allow(clippy::new_ret_no_self)]
    fn new(
        inputs: Inputs,
        outputs: Outputs,
        queries: Queries,
        queryables: Queryables,
        configuration: serde_yml::Value,
    ) -> tokio::task::JoinHandle<Result<Box<dyn Node>>>
    where
        Self: Sized;

    /// The `start` function is used to start the node's execution
    fn start(self: Box<Self>) -> tokio::task::JoinHandle<Result<()>>;
}

/// The `DynamicallyLinkedNodeInstance` type is used for the `C` symbolic function
pub type DynamicallyLinkedNodeInstance = fn(
    Inputs,
    Outputs,
    Queries,
    Queryables,
    serde_yml::Value,
) -> tokio::task::JoinHandle<Result<Box<dyn Node>>>;
