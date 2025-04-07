use std::time::Duration;

use crate::prelude::*;

static TOKIO_RUNTIME: std::sync::LazyLock<tokio::runtime::Runtime> =
    std::sync::LazyLock::new(|| {
        tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime")
    });

fn runtime_spawn<T: Send + 'static>(
    task: impl Future<Output = T> + Send + 'static,
) -> tokio::task::JoinHandle<T> {
    match Handle::try_current() {
        Ok(handle) => handle.spawn(task),
        Err(_) => TOKIO_RUNTIME.spawn(task),
    }
}

pub struct Timer {
    pub output: Output<String>,
    pub frequency: f64,
}

#[node(runtime = "runtime_spawn")]
impl Node for Timer {
    async fn new(
        _: Inputs,
        mut outputs: Outputs,
        configuration: serde_yml::Value,
    ) -> eyre::Result<Box<dyn Node>>
    where
        Self: Sized,
    {
        let frequency = match configuration.get("frequency") {
            Some(serde_yml::Value::Number(number)) => number.as_f64().unwrap_or(1.0),
            _ => 1.0,
        };

        Ok(Box::new(Self {
            output: outputs.with("out").await?,
            frequency,
        }) as Box<dyn Node>)
    }

    async fn start(self: Box<Self>) -> eyre::Result<()> {
        loop {
            tokio::time::sleep(Duration::from_millis((1000.0 / self.frequency) as u64)).await;

            self.output
                .send("tick".to_string())
                .wrap_err("Failed to send message")?;
        }
    }
}
