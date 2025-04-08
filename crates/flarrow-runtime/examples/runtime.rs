use std::sync::Arc;

use flarrow_runtime::prelude::*;

use flarrow_url_default::UrlDefaultPlugin;

use url::Url;

#[derive(Node)]
pub struct MyOperator {
    pub input: Input<String>,
    pub output: Output<String>,

    counter: u32,
}

#[node(runtime = "default_runtime")]
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
            counter: 0,
        }) as Box<dyn Node>)
    }

    async fn start(mut self: Box<Self>) -> eyre::Result<()> {
        while let Ok((_, message)) = self.input.recv_async().await {
            self.counter += 1;

            self.output
                .send(format!("{} - {}", self.counter, message))
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
    let flows = Flows::new(layout.clone(), async move |connector: &mut Connector| {
        connector.connect(op_in, output)?;
        connector.connect(input, op_out)?;

        Ok(())
    })
    .await?;

    let path = std::env::var("CARGO_MANIFEST_DIR")?;
    let examples = format!("file://{}/../../target/debug/examples", path);

    let runtime = DataflowRuntime::new(
        flows,
        Some(
            RuntimeUrlPlugin::new_statically_linked::<UrlDefaultPlugin>()
                .await
                .wrap_err("Failed to load URL plugin")?,
        ),
        async move |loader: &mut Loader| {
            loader
                .load_statically_linked::<MyOperator>(operator, serde_yml::Value::from(""))
                .await
                .wrap_err("Failed to load MyOperator")?;

            let source_file = Url::parse("builtin:///timer")?;
            let sink_file = Url::parse(&format!("{}/libsink.so", examples))?;

            loader
                .load_from_url(source, source_file, serde_yml::from_str("frequency: 5.0")?)
                .await
                .wrap_err("Failed to load source")?;
            loader
                .load_from_url(sink, sink_file, serde_yml::Value::from(""))
                .await
                .wrap_err("Failed to load sink")?;

            Ok(())
        },
    )
    .await?;

    runtime.run().await
}
