use iridis_node::prelude::{thirdparty::*, *};

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
        while let Ok(TypedDataflowMessage {
            header: _,
            data: message,
        }) = self.input.recv().await
        {
            println!("Received message: {}", message);
        }

        Ok(())
    }
}
