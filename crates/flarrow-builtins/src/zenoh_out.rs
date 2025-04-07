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

pub struct ZenohOut {
    pub output: Output<String>,
}

#[node(runtime = "runtime_spawn")]
impl Node for ZenohOut {
    async fn new(
        _: Inputs,
        mut outputs: Outputs,
        _: serde_yml::Value,
    ) -> eyre::Result<Box<dyn Node>>
    where
        Self: Sized,
    {
        Ok(Box::new(Self {
            output: outputs.with("out").await?,
        }) as Box<dyn Node>)
    }

    async fn start(self: Box<Self>) -> eyre::Result<()> {
        loop {
            self.output
                .send("tick".to_string())
                .wrap_err("Failed to send message")?;
        }
    }
}
