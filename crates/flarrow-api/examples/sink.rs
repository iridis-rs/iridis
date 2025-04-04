use flarrow_api::prelude::*;

#[derive(Node)]
pub struct MySink {
    pub input: Input<String>,
}

#[node(runtime = "runtime_spawn")]
impl Node for MySink {
    async fn new(mut inputs: Inputs, _: Outputs, _: YAMLValue) -> Result<Box<dyn Node>>
    where
        Self: Sized,
    {
        Ok(Box::new(Self {
            input: inputs.with("in").await.wrap_err("Failed to create input")?,
        }) as Box<dyn Node>)
    }

    async fn start(mut self: Box<Self>) -> Result<()> {
        while let Ok((_, message)) = self.input.recv_async().await {
            println!("Received message: {}", message);
        }

        Ok(())
    }
}
