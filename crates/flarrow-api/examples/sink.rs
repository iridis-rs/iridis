use flarrow_api::prelude::*;

#[derive(Node)]
pub struct MySink {
    pub input: Input<String>,
}

#[node(runtime = "default_runtime")]
impl Node for MySink {
    async fn new(
        mut inputs: Inputs,
        _: Outputs,
        _: Queries,
        _: Queryables,
        _: serde_yml::Value,
    ) -> Result<Self> {
        Ok(Self {
            input: inputs.with("in").await.wrap_err("Failed to create input")?,
        })
    }

    async fn start(mut self: Box<Self>) -> Result<()> {
        while let Ok((_, message)) = self.input.recv_async().await {
            println!("Received message: {}", message);
        }

        Ok(())
    }
}
