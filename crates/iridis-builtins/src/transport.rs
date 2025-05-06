use crate::prelude::*;

/// Simple operator that does nothing, it just passes its input to its output.
#[derive(Node)]
pub struct Transport {
    pub input: RawInput,
    pub output: RawOutput,
}

#[node(runtime = "default_runtime")]
impl Node for Transport {
    async fn new(
        mut inputs: Inputs,
        mut outputs: Outputs,
        _: Queries,
        _: Queryables,
        _: serde_yml::Value,
    ) -> Result<Self> {
        Ok(Self {
            input: inputs.raw("in").await?,
            output: outputs.raw("out").await?,
        })
    }

    async fn start(mut self: Box<Self>) -> Result<()> {
        while let Ok(DataflowMessage { header: _, data }) = self.input.recv().await {
            self.output.send(data).await?;
        }

        Ok(())
    }
}
