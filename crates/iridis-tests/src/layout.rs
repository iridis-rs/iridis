use iridis::prelude::{thirdparty::*, *};

async fn layout() -> (
    SharedDataLayout,
    PrimitiveID,
    PrimitiveID,
    PrimitiveID,
    PrimitiveID,
    PrimitiveID,
    PrimitiveID,
    PrimitiveID,
    PrimitiveID,
) {
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

    let (_service, (compare_to_128, compare_to_64)) = layout
        .node("service", async |builder: &mut NodeLayout| {
            (
                builder.queryable("compare_to_128"),
                builder.queryable("compare_to_64"),
            )
        })
        .await;

    let (_client, (ask_128, ask_64)) = layout
        .node("client", async |builder: &mut NodeLayout| {
            (builder.query("ask_128"), builder.query("ask_64"))
        })
        .await;

    (
        layout,
        output,
        op_in,
        op_out,
        input,
        compare_to_128,
        compare_to_64,
        ask_128,
        ask_64,
    )
}

#[tokio::test]
async fn create_and_validate_layout() {
    let (layout, output, op_in, op_out, input, compare_to_128, compare_to_64, ask_128, ask_64) =
        layout().await;

    let _layout = layout
        .finish(async |flows| {
            flows.connect(op_in, output)?;
            flows.connect(input, op_out)?;

            flows.connect(ask_128, compare_to_128.clone())?;
            flows.connect(ask_64, compare_to_64.clone())?;

            Ok(())
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn check_flow_order() {
    let (layout, output, op_in, op_out, input, compare_to_128, compare_to_64, ask_128, ask_64) =
        layout().await;

    let _layout = layout
        .finish(async |flows| {
            flows.connect(output, op_in)?;
            flows.connect(input, op_out)?;

            flows.connect(compare_to_128, ask_128)?;
            flows.connect(ask_64, compare_to_64)?;

            Ok(())
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn check_flow_error() {
    let (layout, output, op_in, op_out, input, compare_to_128, compare_to_64, ask_128, ask_64) =
        layout().await;

    let layout = layout
        .finish(async |flows| {
            flows.connect(output, op_in)?;
            flows.connect(input, compare_to_128)?;

            flows.connect(op_out, ask_128)?;
            flows.connect(ask_64, compare_to_64)?;

            Ok(())
        })
        .await;

    assert!(layout.is_err());
}
