use iridis_layout::prelude::{thirdparty::*, *};

#[tokio::main]
async fn main() -> Result<()> {
    let layout = DataflowLayout::empty();

    let (_source, output) = layout
        .node("source", async |builder: &mut NodeLayout| {
            builder.output("out")
        })
        .await;

    let (_operator, (op_in, op_out)) = layout
        .node("operator", async |builder: &mut NodeLayout| {
            (builder.input("in"), builder.output("out"))
        })
        .await;

    let (_sink, input) = layout
        .node("sink", async |builder: &mut NodeLayout| builder.input("in"))
        .await;

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
