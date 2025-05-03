use iridis_runtime::prelude::{thirdparty::*, *};

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

    runtime
        .run(flows, async move |loader: &mut NodeLoader| {
            loader
                .load::<Timer>(source, serde_yml::from_str("frequency: 1.0")?)
                .await?;

            loader
                .load::<Transport>(operator, serde_yml::from_str("")?)
                .await?;

            loader
                .load::<Printer>(sink, serde_yml::from_str("")?)
                .await?;

            Ok(())
        })
        .await
}
