use iridis_layout::prelude::{thirdparty::*, *};

#[tokio::main]
async fn main() -> Result<()> {
    let mut layout = DataflowLayout::new();

    let (_source, _output) = layout
        .node("source", async |builder: &mut NodeIOBuilder| {
            builder.output("out")
        })
        .await;

    let (_operator, (_op_in, _op_out)) = layout
        .node("operator", async |builder: &mut NodeIOBuilder| {
            (builder.input("in"), builder.output("out"))
        })
        .await;

    let (_sink, _input) = layout
        .node("sink", async |builder: &mut NodeIOBuilder| {
            builder.input("in")
        })
        .await;

    let layout = layout.build();

    println!("{:?}", layout);

    Ok(())
}
