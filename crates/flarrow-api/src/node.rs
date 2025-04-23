use tokio::task::JoinHandle;

use crate::prelude::*;

pub enum NodeResult {
    Nothing,
}

pub trait Node: Send + Sync {
    #[allow(clippy::new_ret_no_self)]
    fn new(
        inputs: Inputs,
        outputs: Outputs,
        queries: Queries,
        queryables: Queryables,
        configuration: serde_yml::Value,
    ) -> NodeNewResult
    where
        Self: Sized;

    fn start(self: Box<Self>) -> NodeStartResult;
}

pub type NodeNewResult = JoinHandle<eyre::Result<Box<dyn Node>>>;
pub type NodeStartResult = JoinHandle<eyre::Result<()>>;

pub type DynamicallyLinkedNodeInstance =
    fn(Inputs, Outputs, Queries, Queryables, serde_yml::Value) -> NodeNewResult;
