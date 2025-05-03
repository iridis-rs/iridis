use iridis_layout::prelude::{thirdparty::*, *};

#[tokio::main]
async fn main() -> Result<()> {
    let mut layout = DataflowLayout::new();

    let (_service, _queryable) = layout
        .node("service", async |builder: &mut NodeIOBuilder| {
            builder.queryable("queryable")
        })
        .await;

    let (_client, _query) = layout
        .node("client", async |builder: &mut NodeIOBuilder| {
            builder.query("query")
        })
        .await;

    let layout = layout.build();

    println!("{:?}", layout);

    Ok(())
}
