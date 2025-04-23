use std::collections::HashSet;

use crate::prelude::*;

pub struct DataflowLayout {
    pub inputs: HashSet<InputUUID>,
    pub outputs: HashSet<OutputUUID>,
    pub queryables: HashSet<QueryableUUID>,
    pub queries: HashSet<QueryUUID>,
}

impl Default for DataflowLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl DataflowLayout {
    pub fn new() -> Self {
        Self {
            inputs: HashSet::new(),
            outputs: HashSet::new(),
            queryables: HashSet::new(),
            queries: HashSet::new(),
        }
    }

    pub async fn create_node<T>(
        &mut self,
        node: impl AsyncFnOnce(&mut NodeIO) -> T,
    ) -> (NodeUUID, T) {
        let mut io = NodeIO::new();
        let result = node(&mut io).await;

        self.inputs.extend(io.inputs);
        self.outputs.extend(io.outputs);
        self.queryables.extend(io.queryables);
        self.queries.extend(io.queries);

        (io.id, result)
    }
}
