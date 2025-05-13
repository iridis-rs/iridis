use std::time::Duration;

use iridis::prelude::{thirdparty::*, *};

#[tokio::main]
async fn main() -> Result<()> {
    let layout = DataflowLayout::empty();

    let source = layout.clone();
    let source = tokio::spawn(async move {
        // simulate some work, like reading from a file, requesting data from a network, etc.
        tokio::time::sleep(Duration::from_millis(200)).await;

        source
            .node("source", async |builder: &mut NodeLayout| {
                builder.output("out")
            })
            .await
    });

    let other = layout.clone();
    let other = tokio::spawn(async move {
        // simulate some work, like reading from a file, requesting data from a network, etc.
        tokio::time::sleep(Duration::from_millis(100)).await;

        let (operator, (op_in, op_out)) = other
            .node("operator", async |builder: &mut NodeLayout| {
                (builder.input("in"), builder.output("out"))
            })
            .await;

        let (sink, input) = other
            .node("sink", async |builder: &mut NodeLayout| builder.input("in"))
            .await;

        (operator, op_in, op_out, sink, input)
    });

    let (_source, output) = source.await?;
    let (_operator, op_in, op_out, _sink, input) = other.await?;

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
