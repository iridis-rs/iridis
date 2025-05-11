# Messages

`iridis` nodes communicate with each other using the `Arrow` data-format. This allows for efficient communication without copying the data.

To create `Arrow` messages, the `iridis-message` crate defines a `trait` and a `derive` procedure to automatically implement the `trait` for your own types.

The `ArrowMessage` trait is implemented for all `rust` primitives (including `Option` and `Enums` with no values). It is also implemented for all `arrow arrys` such as `UInt8Array`, `Float64Array`, `StringArray`, etc.

Then to create a new `ArrowMessage`, you can use the macro:

```rust
#[derive(ArrowMessage)]
struct MyMessage {}
```

Each field should then implement the `ArrowMessage` trait.

```rust
use iridis_message::prelude::{
    thirdparty::{arrow_array::*, *},
    *,
};

#[derive(Debug, ArrowMessage)]
struct Metadata {
    name: Option<String>,
    width: u32,
    height: u32,
}

#[derive(Debug, ArrowMessage)]
struct Image {
    data: UInt8Array,
    metadata: Option<Metadata>,
}
```

Then you can create messages without copy, using the `Arrow` data-format:

```rust
use iridis_message::prelude::{
    thirdparty::{arrow_array::*, arrow_data::*, *},
    *,
};

let image = Image {
    data: UInt8Array::from(vec![1, 2, 3]),
    metadata: Some(Metadata {
        name: Some("example".to_string()),
        width: 12,
        height: 12,
    }),
};

let arrow = ArrayData::try_from(image)?; // No copy made
let image = Image::try_from(arrow)?; // Again, no copy made, this is the same underlying buffer
```
