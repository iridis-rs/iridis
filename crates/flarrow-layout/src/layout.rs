use std::collections::HashSet;

use crate::prelude::*;

pub struct DataflowLayout {
    pub inputs: HashSet<InputID>,
    pub outputs: HashSet<OutputID>,
}

impl DataflowLayout {
    pub fn new() -> Self {
        Self {
            inputs: HashSet::new(),
            outputs: HashSet::new(),
        }
    }

    pub async fn create_node<T>(&mut self, node: impl AsyncFn(&mut NodeIO) -> T) -> (NodeID, T) {
        let mut io = NodeIO::new();
        let result = node(&mut io).await;

        self.inputs.extend(io.inputs);
        self.outputs.extend(io.outputs);

        (io.id, result)
    }
}
