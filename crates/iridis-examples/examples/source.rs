use std::time::Duration;

use iridis_node::prelude::{thirdparty::*, *};

#[derive(Node)]
pub struct MySource {
    pub output: Output<String>,
}

#[node(runtime = "default_runtime")]
impl Node for MySource {
    async fn new(
        _: Inputs,
        mut outputs: Outputs,
        _: Queries,
        _: Queryables,
        _: serde_yml::Value,
    ) -> Result<Self> {
        Ok(Self {
            output: outputs
                .with("out")
                .await
                .wrap_err("Failed to create output")?,
        })
    }

    async fn start(self: Box<Self>) -> Result<()> {
        while let Ok(()) = self
            .output
            .send("tick".to_string())
            .await
            .wrap_err("Failed to send message")
        {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        Ok(())
    }
}
