use iridis::prelude::{thirdparty::*, *};

#[tokio::main]
async fn main() -> Result<()> {
    let mut layout = DataflowLayout::new();

    let (_source, output) = layout
        .node("source", async |builder: &mut Builder| {
            builder.output("out")
        })
        .await;

    let (_operator, (op_in, op_out)) = layout
        .node("operator", async |builder: &mut Builder| {
            (builder.input("in"), builder.output("out"))
        })
        .await;

    let (_sink, input) = layout
        .node("sink", async |builder: &mut Builder| builder.input("in"))
        .await;

    let layout = layout.build();

    let _flows = Flows::new(layout.clone(), async move |flows: &mut Connector| {
        flows.connect(op_in, output, None)?;
        flows.connect(input, op_out, None)?;

        Ok(())
    })
    .await?;

    Ok(())
}
