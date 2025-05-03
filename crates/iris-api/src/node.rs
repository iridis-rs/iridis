use crate::prelude::*;

pub trait Node: Send + Sync {
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

    fn start(self: Box<Self>) -> tokio::task::JoinHandle<Result<()>>;
}

pub type DynamicallyLinkedNodeInstance = fn(
    Inputs,
    Outputs,
    Queries,
    Queryables,
    serde_yml::Value,
) -> tokio::task::JoinHandle<Result<Box<dyn Node>>>;
