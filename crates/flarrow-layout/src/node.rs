use std::collections::HashSet;

use uuid::Uuid;

use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeUUID(pub Uuid);

impl Default for NodeUUID {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeUUID {
    pub fn new() -> Self {
        NodeUUID(Uuid::new_v4())
    }

    pub fn input(&self, input: impl Into<String>) -> InputUUID {
        InputUUID(Uuid::new_v3(&self.0, input.into().as_bytes()))
    }

    pub fn output(&self, output: impl Into<String>) -> OutputUUID {
        OutputUUID(Uuid::new_v3(&self.0, output.into().as_bytes()))
    }

    pub fn queryable(&self, queryable: impl Into<String>) -> QueryableUUID {
        QueryableUUID(Uuid::new_v3(&self.0, queryable.into().as_bytes()))
    }

    pub fn query(&self, query: impl Into<String>) -> QueryUUID {
        QueryUUID(Uuid::new_v3(&self.0, query.into().as_bytes()))
    }
}

pub struct NodeIO {
    pub id: NodeUUID,

    pub inputs: HashSet<InputUUID>,
    pub outputs: HashSet<OutputUUID>,
    pub queryables: HashSet<QueryableUUID>,
    pub queries: HashSet<QueryUUID>,
}

impl Default for NodeIO {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeIO {
    pub fn new() -> Self {
        Self {
            id: NodeUUID::new(),
            inputs: HashSet::new(),
            outputs: HashSet::new(),
            queryables: HashSet::new(),
            queries: HashSet::new(),
        }
    }

    pub fn open_input(&mut self, input: impl Into<String>) -> InputUUID {
        let input_id = self.id.input(input);

        self.inputs.insert(input_id);

        input_id
    }

    pub fn open_output(&mut self, output: impl Into<String>) -> OutputUUID {
        let output_id = self.id.output(output);

        self.outputs.insert(output_id);

        output_id
    }

    pub fn open_queryable(&mut self, queryable: impl Into<String>) -> QueryableUUID {
        let queryable_id = self.id.queryable(queryable);

        self.queryables.insert(queryable_id);

        queryable_id
    }

    pub fn open_query(&mut self, query: impl Into<String>) -> QueryUUID {
        let query_id = self.id.query(query);

        self.queries.insert(query_id);

        query_id
    }
}
