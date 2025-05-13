# Layout and Flows

An application is essentially a 'graph' of nodes. Where nodes can be seen as a 'function' that takes an input and produces an output or a standalone process that will just produce or consume data. The nodes are connected together by 'flows' that describe how the data is passed from one node to another.

## Layout

It's really important when making a `dataflow` application to prepare the layout of the application before even writing the code. You can draw your application on paper and then translate it to code.

The first step is to create a `DataflowLayout` that describes the number of nodes, their names and their inputs/outputs. This is done by using the `node` method of the `DataflowLayout` struct. The `node` method takes a name and a closure.

```rust
use iridis_layout::prelude::{thirdparty::*, *};

let mut layout = DataflowLayout::new();

let (_source, output) = layout
    .node("source", async |builder: &mut Builder| {
        builder.output("out")
    })
    .await;
```

The `node` method will create a new node in the layout with the given name and will always return at least the associated `NodeID` object (which is just a label together with an uuid to represent this node). Inside the closure, you have access to a `Builder` that can add inputs, outputs and queries to the node. You can return anything you want from the closure, but it is recommended to return the result of the `builder` methods. This will allow you to use the `PrimitiveID` object later on to create the flows.

## Flows

Once you have create the layout, it's recommended to build it as an immutable shared object:

```rust
let layout = layout.build();
```

Then you can create the connections (the flows) for this layout. This is done by using the `Flows` struct. The `Flows` struct takes a `DataflowLayout` and a closure that will be called to create the flows.

```rust
use iridis_flows::prelude::{thirdparty::*, *};

let flows = Flows::new(layout.clone(), async move |flows: &mut Connector| {
    flows.connect(op_in, output, None)?;
    flows.connect(input, op_out, None)?;

    Ok(())
})
.await?;
```

Note that the `connect` method will recognize each IO kind, so you don't have to worry about the order between the input and output, the query and the queryable. The `connect` method will also take care of the creation of the communication channels, and so you can adjust the `capacity` of the channel by passing a `Some(capacity)` value as the last parameter. If you pass `None`, the default capacity will be used (128).
