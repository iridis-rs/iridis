use iridis::prelude::{thirdparty::*, *};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let mut layout = DataflowLayout::new();

    let (source, output) = layout
        .node("source", async |builder: &mut NodeIOBuilder| {
            builder.output("out")
        })
        .await;

    let (operator, (op_in, op_out)) = layout
        .node("operator", async |builder: &mut NodeIOBuilder| {
            (builder.input("in"), builder.output("out"))
        })
        .await;

    let (sink, input) = layout
        .node("sink", async |builder: &mut NodeIOBuilder| {
            builder.input("in")
        })
        .await;

    let layout = layout.build();

    let flows = Flows::new(layout.clone(), async move |builder: &mut FlowsBuilder| {
        builder.connect(op_in, output, None)?;
        builder.connect(input, op_out, None)?;

        Ok(())
    })
    .await?;

    let runtime = Runtime::new(
        async |_file_ext: &mut FileExtManagerBuilder, _url_scheme: &mut UrlSchemeManagerBuilder| {
            Ok(())
        },
    )
    .await?;

    let path = std::env::var("CARGO_MANIFEST_DIR")?;
    let examples = format!("file://{}/../../target/debug/examples", path);

    let prefix = std::env::consts::DLL_PREFIX;
    let suffix = std::env::consts::DLL_SUFFIX;

    runtime
        .run(flows, async move |loader: &mut NodeLoader| {
            let source_file = Url::parse(&format!("{}/{}source{}", examples, prefix, suffix))?;
            let sink_file = Url::parse(&format!("{}/{}sink{}", examples, prefix, suffix))?;

            loader
                .load_url(source_file, source, serde_yml::from_str("")?)
                .await?;

            loader
                .load::<Transport>(operator, serde_yml::from_str("")?)
                .await?;

            loader
                .load_url(sink_file, sink, serde_yml::from_str("")?)
                .await?;

            Ok(())
        })
        .await
}
