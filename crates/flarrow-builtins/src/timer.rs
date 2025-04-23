use std::time::Duration;

use crate::prelude::*;

static TOKIO_RUNTIME: std::sync::LazyLock<tokio::runtime::Runtime> =
    std::sync::LazyLock::new(|| {
        tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime")
    });

fn default_runtime<T: Send + 'static>(
    task: impl Future<Output = T> + Send + 'static,
) -> tokio::task::JoinHandle<T> {
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => handle.spawn(task),
        Err(_) => TOKIO_RUNTIME.spawn(task),
    }
}

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
        loop {
            tokio::time::sleep(Duration::from_millis((1000.0 / self.frequency) as u64)).await;

            self.output
                .send_async("tick".to_string())
                .await
                .wrap_err("Failed to send message")?;
        }
    }
}
