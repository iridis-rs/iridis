use std::collections::HashSet;

use flarrow_layout::prelude::*;

#[tokio::main]
async fn main() {
    let mut layout = DataflowLayout::new();

    let (source, service) = layout
        .create_node(async |io: &mut NodeIO| io.open_queryable("service"))
        .await;

    let (sink, client) = layout
        .create_node(async |io: &mut NodeIO| io.open_query("client"))
        .await;

    /// Convenient struct for printing the layout of this example
    #[derive(Debug)]
    struct Node {
        _id: (String, NodeUUID),
        _inputs: HashSet<(String, InputUUID)>,
        _outputs: HashSet<(String, OutputUUID)>,
        _queryables: HashSet<(String, QueryableUUID)>,
        _queries: HashSet<(String, QueryUUID)>,
    }

    let nodes = vec![
        Node {
            _id: (String::from("source"), source),
            _inputs: HashSet::new(),
            _outputs: HashSet::new(),
            _queryables: HashSet::from([(String::from("service"), service)]),
            _queries: HashSet::new(),
        },
        Node {
            _id: (String::from("sink"), sink),
            _inputs: HashSet::new(),
            _outputs: HashSet::new(),
            _queryables: HashSet::new(),
            _queries: HashSet::from([(String::from("client"), client)]),
        },
    ];

    println!("{:#?}", nodes);
}
