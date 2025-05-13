use std::time::Duration;

use iridis_layout::prelude::{thirdparty::*, *};

#[tokio::main]
async fn main() -> Result<()> {
    let layout = DataflowLayout::empty();

    let source = layout.clone();
    let source = tokio::spawn(async move {
        source
            .node("source", async |builder: &mut NodeLayout| {
                tokio::time::sleep(Duration::from_secs(1)).await;
                builder.output("out")
            })
            .await
    });

    let operator = layout.clone();
    let operator = tokio::spawn(async move {
        operator
            .node("operator", async |builder: &mut NodeLayout| {
                tokio::time::sleep(Duration::from_secs(1)).await;
                (builder.input("in"), builder.output("out"))
            })
            .await
    });

    let sink = layout.clone();
    let sink = tokio::spawn(async move {
        sink.node("sink", async |builder: &mut NodeLayout| {
            tokio::time::sleep(Duration::from_secs(1)).await;
            builder.input("in")
        })
        .await
    });

    let (_source, output) = source.await?;
    let (_operator, (op_in, op_out)) = operator.await?;
    let (_sink, input) = sink.await?;

    let layout = layout
        .finish(async |flows| {
            flows.connect(op_in, output)?;
            flows.connect(input, op_out)?;

            Ok(())
        })
        .await?;

    println!("{:#?}", layout);

    Ok(())
}
