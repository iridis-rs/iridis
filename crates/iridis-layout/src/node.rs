use std::collections::{HashMap, HashSet};

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeID {
    pub label: String,
    pub uuid: Uuid,
}

impl NodeID {
    pub fn new(label: impl Into<String>) -> Self {
        NodeID {
            label: label.into(),
            uuid: Uuid::new_v4(),
        }
    }

    pub fn input(&self, input: impl Into<String>) -> InputID {
        let label = input.into();

        InputID {
            uuid: Uuid::new_v3(&self.uuid, label.as_bytes()),
            label,
        }
    }

    pub fn output(&self, output: impl Into<String>) -> OutputID {
        let label = output.into();

        OutputID {
            uuid: Uuid::new_v3(&self.uuid, label.as_bytes()),
            label,
        }
    }

    pub fn query(&self, query: impl Into<String>) -> QueryID {
        let label = query.into();

        QueryID {
            uuid: Uuid::new_v3(&self.uuid, label.as_bytes()),
            label,
        }
    }

    pub fn queryable(&self, queryable: impl Into<String>) -> QueryableID {
        let label = queryable.into();

        QueryableID {
            uuid: Uuid::new_v3(&self.uuid, label.as_bytes()),
            label,
        }
    }
}

pub struct NodeDataLayout {
    pub inputs: HashSet<Uuid>,
    pub outputs: HashSet<Uuid>,
    pub queries: HashSet<Uuid>,
    pub queryables: HashSet<Uuid>,
}

pub struct NodeDebugLayout {
    pub labels: HashMap<Uuid, String>,
}

pub struct NodeLayout {
    pub id: NodeID,
    pub data: NodeDataLayout,
    pub debug: NodeDebugLayout,
}

impl NodeLayout {
    pub fn new(id: &NodeID) -> Self {
        Self {
            id: id.clone(),
            data: NodeDataLayout {
                inputs: HashSet::new(),
                outputs: HashSet::new(),
                queries: HashSet::new(),
                queryables: HashSet::new(),
            },
            debug: NodeDebugLayout {
                labels: HashMap::new(),
            },
        }
    }

    pub fn input(&mut self, input: impl Into<String>) -> PrimitiveID {
        let label: String = input.into();
        let layout = self.id.input(&label);

        self.data.inputs.insert(layout.uuid);

        self.debug.labels.insert(layout.uuid, label.clone());

        tracing::debug!(
            "Node '{}' (uuid: {}) added input layout with label: '{}' (uuid: {})",
            self.id.label,
            self.id.uuid,
            label,
            layout.uuid
        );

        layout.into()
    }

    pub fn output(&mut self, output: impl Into<String>) -> PrimitiveID {
        let label: String = output.into();
        let layout = self.id.output(&label);

        self.data.outputs.insert(layout.uuid);

        self.debug.labels.insert(layout.uuid, label.clone());

        tracing::debug!(
            "Node '{}' (uuid: {}) added output layout with label: '{}' (uuid: {})",
            self.id.label,
            self.id.uuid,
            label,
            layout.uuid
        );

        layout.into()
    }

    pub fn query(&mut self, query: impl Into<String>) -> PrimitiveID {
        let label: String = query.into();
        let layout = self.id.query(&label);

        self.data.queries.insert(layout.uuid);

        self.debug.labels.insert(layout.uuid, label.clone());

        tracing::debug!(
            "Node '{}' (uuid: {}) added query layout with label: '{}' (uuid: {})",
            self.id.label,
            self.id.uuid,
            label,
            layout.uuid
        );

        layout.into()
    }

    pub fn queryable(&mut self, queryable: impl Into<String>) -> PrimitiveID {
        let label: String = queryable.into();
        let layout = self.id.queryable(&label);

        self.data.queryables.insert(layout.uuid);

        self.debug.labels.insert(layout.uuid, label.clone());

        tracing::debug!(
            "Node '{}' (uuid: {}) added queryable layout with label: '{}' (uuid: {})",
            self.id.label,
            self.id.uuid,
            label,
            layout.uuid
        );

        layout.into()
    }
}
