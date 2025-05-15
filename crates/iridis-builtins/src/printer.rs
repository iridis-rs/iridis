//! This module contains the built-in `Printer` node, which is a simple
//! sink node that just prints its input

use crate::prelude::*;

/// Simple sink node that just prints its input to stdout.
#[derive(Node)]
pub struct Printer {
    pub input: RawInput,
}

#[node(runtime = "default_runtime")]
impl Node for Printer {
    async fn new(
        mut inputs: Inputs,
        _: Outputs,
        _: Queries,
        _: Queryables,
        _: serde_yml::Value,
    ) -> Result<Self> {
        Ok(Self {
            input: inputs.raw("in").await?,
        })
    }

    async fn start(mut self: Box<Self>) -> Result<()> {
        while let Ok(msg) = self.input.recv().await {
            println!("{:?}", msg);
        }

        Ok(())
    }
}
