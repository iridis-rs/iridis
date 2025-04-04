use flarrow_api::prelude::*;

#[derive(Node)]
pub struct MySource {
    pub output: Output<String>,
}

#[node(runtime = "runtime_spawn")]
impl Node for MySource {
    async fn new(_: Inputs, mut outputs: Outputs, _: YAMLValue) -> Result<Box<dyn Node>>
    where
        Self: Sized,
    {
        Ok(Box::new(Self {
            output: outputs
                .with("out")
                .await
                .wrap_err("Failed to create output")?,
        }) as Box<dyn Node>)
    }

    async fn start(self: Box<Self>) -> Result<()> {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;

            self.output
                .send("Hello, world!".to_string())
                .wrap_err("Failed to send message")?;
        }
    }
}
