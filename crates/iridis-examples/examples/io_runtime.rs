use iridis::prelude::{thirdparty::*, *};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let mut layout = DataflowLayout::new();

    let (source, output) = layout
        .node("source", async |builder: &mut Builder| {
            builder.output("out")
        })
        .await;

    let (operator, (op_in, op_out)) = layout
        .node("operator", async |builder: &mut Builder| {
            (builder.input("in"), builder.output("out"))
        })
        .await;

    let (sink, input) = layout
        .node("sink", async |builder: &mut Builder| builder.input("in"))
        .await;

    let layout = layout.build();

    let flows = Flows::new(layout.clone(), async move |flows: &mut Connector| {
        flows.connect(op_in, output, None)?;
        flows.connect(input, op_out, None)?;

        Ok(())
    })
    .await?;

    let runtime = Runtime::new(
        async |_file_ext: &mut FileExtLoader, _url_scheme: &mut UrlSchemeLoader| Ok(()),
    )
    .await?;

    runtime
        .run(flows, async move |loader: &mut Loader| {
            loader
                .load_url(
                    iridis_examples::dylib("source", None)?,
                    source,
                    serde_yml::from_str("")?,
                )
                .await?;

            loader
                .load::<Transport>(operator, serde_yml::from_str("")?)
                .await?;

            loader
                .load_url(
                    iridis_examples::dylib("sink", None)?,
                    sink,
                    serde_yml::from_str("")?,
                )
                .await?;

            Ok(())
        })
        .await
}
