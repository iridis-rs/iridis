use std::time::Duration;

use crate::prelude::*;

/// Simple source node that emits a "tick" message at a specified frequency.
#[derive(Node)]
pub struct Timer {
    pub output: Output<String>,
    pub frequency: f64,
}

#[node(runtime = "default_runtime")]
impl Node for Timer {
    async fn new(
        _: Inputs,
        mut outputs: Outputs,
        _: Queries,
        _: Queryables,
        configuration: serde_yml::Value,
    ) -> Result<Self> {
        let frequency = match configuration.get("frequency") {
            Some(serde_yml::Value::Number(number)) => number.as_f64().unwrap_or(1.0),
            _ => 1.0,
        };

        Ok(Self {
            output: outputs.with("out").await?,
            frequency,
        })
    }

    async fn start(self: Box<Self>) -> Result<()> {
        while let Ok(()) = self
            .output
            .send("tick".to_string())
            .await
            .wrap_err("Failed to send message")
        {
            tokio::time::sleep(Duration::from_millis((1000.0 / self.frequency) as u64)).await;
        }

        Ok(())
    }
}
