use iridis::prelude::{thirdparty::*, *};

#[tokio::main]
async fn main() -> Result<()> {
    let layout = DataflowLayout::empty();

    let (_service, queryable) = layout
        .node("service", async |builder: &mut NodeLayout| {
            builder.queryable("queryable")
        })
        .await;

    let (_client, query) = layout
        .node("client", async |builder: &mut NodeLayout| {
            builder.query("query")
        })
        .await;

    let layout = layout
        .finish(async move |flows| {
            flows.connect(query, queryable)?;

            Ok(())
        })
        .await?;

    println!("{:#?}", layout);

    Ok(())
}
