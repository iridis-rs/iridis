use flarrow_layout::prelude::*;

use flarrow_flows::prelude::{thirdparty::*, *};

#[tokio::main]
async fn main() -> Result<()> {
    let mut layout = DataflowLayout::new();

    let (_service, queryable) = layout
        .node("service", async |builder: &mut NodeIOBuilder| {
            builder.queryable("queryable")
        })
        .await;

    let (_client, query) = layout
        .node("client", async |builder: &mut NodeIOBuilder| {
            builder.query("query")
        })
        .await;

    let layout = layout.build();

    let _flows = Flows::new(layout.clone(), async move |builder: &mut FlowsBuilder| {
        builder.connect(query, queryable, None)?;

        Ok(())
    })
    .await?;

    Ok(())
}
