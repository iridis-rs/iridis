use std::collections::HashSet;

use uuid::Uuid;

use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeID(pub Uuid);

impl NodeID {
    pub fn new() -> Self {
        NodeID(Uuid::new_v4())
    }

    pub fn input(&self, input: impl Into<String>) -> InputID {
        InputID(Uuid::new_v3(&self.0, input.into().as_bytes()))
    }

    pub fn output(&self, output: impl Into<String>) -> OutputID {
        OutputID(Uuid::new_v3(&self.0, output.into().as_bytes()))
    }
}

pub struct NodeIO {
    pub id: NodeID,

    pub inputs: HashSet<InputID>,
    pub outputs: HashSet<OutputID>,
}

impl NodeIO {
    pub fn new() -> Self {
        Self {
            id: NodeID::new(),
            inputs: HashSet::new(),
            outputs: HashSet::new(),
        }
    }

    pub fn open_input(&mut self, input: impl Into<String>) -> InputID {
        let input_id = self.id.input(input);

        self.inputs.insert(input_id);

        input_id
    }

    pub fn open_output(&mut self, output: impl Into<String>) -> OutputID {
        let output_id = self.id.output(output);

        self.outputs.insert(output_id);

        output_id
    }
}
