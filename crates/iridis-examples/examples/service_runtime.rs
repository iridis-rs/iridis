use iridis::prelude::{thirdparty::*, *};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let layout = DataflowLayout::empty();

    let (service, (compare_to_128, compare_to_64)) = layout
        .node("service", async |builder: &mut NodeLayout| {
            (
                builder.queryable("compare_to_128"),
                builder.queryable("compare_to_64"),
            )
        })
        .await;

    let (client, (ask_128, ask_64)) = layout
        .node("client", async |builder: &mut NodeLayout| {
            (builder.query("ask_128"), builder.query("ask_64"))
        })
        .await;

    let layout = layout
        .finish(async move |flows| {
            flows.connect(ask_128, compare_to_128.clone())?;
            flows.connect(ask_64, compare_to_64.clone())?;

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
                iridis_examples::dylib("service", None)?,
                service,
                serde_yml::from_str("")?,
            )?;

            loader.load_url(
                iridis_examples::dylib("client", None)?,
                client,
                serde_yml::from_str("")?,
            )?;

            Ok(())
        })
        .await
}
