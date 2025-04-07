use std::sync::Arc;

use flarrow_runtime::prelude::*;
use url::Url;

#[derive(Node)]
pub struct MyOperator {
    pub input: Input<String>,
    pub output: Output<String>,
}

#[node(runtime = "runtime_spawn")]
impl Node for MyOperator {
    async fn new(
        mut inputs: Inputs,
        mut outputs: Outputs,
        _: serde_yml::Value,
    ) -> eyre::Result<Box<dyn Node>>
    where
        Self: Sized,
    {
        Ok(Box::new(Self {
            input: inputs.with("in").await?,
            output: outputs.with("out").await?,
        }) as Box<dyn Node>)
    }

    async fn start(mut self: Box<Self>) -> eyre::Result<()> {
        while let Ok((_, message)) = self.input.recv_async().await {
            self.output
                .send(message)
                .wrap_err("Failed to send message")?;
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut layout = DataflowLayout::new();

    let (source, output) = layout
        .create_node(async |io: &mut NodeIO| io.open_output("out"))
        .await;

    let (operator, (op_in, op_out)) = layout
        .create_node(async |io: &mut NodeIO| (io.open_input("in"), io.open_output("out")))
        .await;

    let (sink, input) = layout
        .create_node(async |io: &mut NodeIO| io.open_input("in"))
        .await;

    let layout = Arc::new(layout);
    let flows = Flows::new(layout.clone(), async move |connectors: &mut Connectors| {
        connectors.connect(op_in, output)?;
        connectors.connect(input, op_out)?;

        Ok(())
    })
    .await?;

    let path = std::env::var("CARGO_MANIFEST_DIR")?;
    let examples = format!("file://{}/../../target/debug/examples", path);

    let runtime = DataflowRuntime::new(flows, async move |loader: &mut Loader| {
        loader
            .load_statically_linked::<MyOperator>(operator, serde_yml::Value::from(""))
            .await
            .wrap_err("Failed to load MyOperator")?;

        let source_file = Url::parse("builtin:///timer")?;
        let sink_file = Url::parse(&format!("{}/libsink.so", examples))?;

        loader
            .load_from_url(source, source_file, serde_yml::from_str("frequency: 2.0")?)
            .await
            .wrap_err("Failed to load source")?;
        loader
            .load_from_url(sink, sink_file, serde_yml::Value::from(""))
            .await
            .wrap_err("Failed to load sink")?;

        Ok(())
    })
    .await?;

    runtime.run().await
}
