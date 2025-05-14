use iridis::prelude::{thirdparty::*, *};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let layout = DataflowLayout::empty();

    let (source, output) = layout
        .node("source", async |builder: &mut NodeLayout| {
            builder.output("out")
        })
        .await;

    let (operator, (op_in, op_out)) = layout
        .node("operator", async |builder: &mut NodeLayout| {
            (builder.input("in"), builder.output("out"))
        })
        .await;

    let (sink, input) = layout
        .node("sink", async |builder: &mut NodeLayout| builder.input("in"))
        .await;

    let layout = layout
        .finish(async |flows| {
            flows.connect(op_in, output)?;
            flows.connect(input, op_out)?;

            Ok(())
        })
        .await?;

    let runtime = Runtime::new(
        async |_file_ext: &mut FileExtLoader, _url_scheme: &mut UrlSchemeLoader| Ok(()),
    )
    .await?;

    runtime
        .run(layout, async move |loader: &mut Loader| {
            loader.load_url(
                iridis_examples::dylib("source", None)?,
                source,
                serde_yml::from_str("")?,
            );

            loader.load::<Transport>(operator, serde_yml::from_str("")?);

            loader.load_url(
                iridis_examples::dylib("sink", None)?,
                sink,
                serde_yml::from_str("")?,
            );

            Ok(())
        })
        .await
}
