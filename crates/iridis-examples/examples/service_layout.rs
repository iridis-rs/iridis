use iridis::prelude::{thirdparty::*, *};

#[tokio::main]
async fn main() -> Result<()> {
    let mut layout = DataflowLayout::new();

    let (_service, queryable) = layout
        .node("service", async |builder: &mut Builder| {
            builder.queryable("queryable")
        })
        .await;

    let (_client, query) = layout
        .node("client", async |builder: &mut Builder| {
            builder.query("query")
        })
        .await;

    let layout = layout.build();

    let _flows = Flows::new(layout.clone(), async move |flows: &mut Connector| {
        flows.connect(query, queryable, None)?;

        Ok(())
    })
    .await?;

    Ok(())
}
