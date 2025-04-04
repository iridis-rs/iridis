use std::collections::HashSet;

use flarrow_layout::prelude::*;

#[tokio::main]
async fn main() {
    let mut layout = DataflowLayout::new();

    let (source, output) = layout
        .create_node(async |io: &mut NodeIO| io.open_output("out"))
        .await;

    let (operator, (op_in, op_out)) = layout
        .create_node(async |io: &mut NodeIO| (io.open_input("in"), io.open_output("out")))
        .await;

    let (sink, input) = layout
        .create_node(async |io: &mut NodeIO| io.open_input("in"))
        .await;

    /// Convenient struct for printing the layout of this example
    #[derive(Debug)]
    struct Node {
        _id: (String, NodeID),
        _inputs: HashSet<(String, InputID)>,
        _outputs: HashSet<(String, OutputID)>,
    }

    let nodes = vec![
        Node {
            _id: (String::from("source"), source),
            _inputs: HashSet::new(),
            _outputs: HashSet::from([(String::from("out"), output)]),
        },
        Node {
            _id: (String::from("operator"), operator),
            _inputs: HashSet::from([(String::from("in"), op_in)]),
            _outputs: HashSet::from([(String::from("out"), op_out)]),
        },
        Node {
            _id: (String::from("sink"), sink),
            _inputs: HashSet::from([(String::from("in"), input)]),
            _outputs: HashSet::new(),
        },
    ];

    println!("{:#?}", nodes);
}
