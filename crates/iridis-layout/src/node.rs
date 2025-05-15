//! This module defines the layout of everything related to a `Node`.

use std::collections::{HashMap, HashSet};

use crate::prelude::*;

/// A unique identifier for a node in the graph, it's
/// composed of a label and a UUID. By design the only thing
/// the runtime needs to know about a node is its UUID. But for
/// debugging purposes, we also keep the label.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeID {
    pub label: String,
    pub uuid: Uuid,
}

impl NodeID {
    /// Creates a new `NodeID` with a random UUID and the given label.
    pub fn new(label: impl Into<String>) -> Self {
        NodeID {
            label: label.into(),
            uuid: Uuid::new_v4(),
        }
    }

    /// Creates a new `InputID` with the given label. The UUID is
    /// generated from its label and the node's UUID.
    pub fn input(&self, input: impl Into<String>) -> InputID {
        let label = input.into();

        InputID {
            uuid: Uuid::new_v3(&self.uuid, label.as_bytes()),
            label,
        }
    }

    /// Creates a new `OutputID` with the given label. The UUID is
    /// generated from its label and the node's UUID.
    pub fn output(&self, output: impl Into<String>) -> OutputID {
        let label = output.into();

        OutputID {
            uuid: Uuid::new_v3(&self.uuid, label.as_bytes()),
            label,
        }
    }

    /// Creates a new `QueryID` with the given label. The UUID is
    /// generated from its label and the node's UUID.
    pub fn query(&self, query: impl Into<String>) -> QueryID {
        let label = query.into();

        QueryID {
            uuid: Uuid::new_v3(&self.uuid, label.as_bytes()),
            label,
        }
    }

    /// Creates a new `QueryableID` with the given label. The UUID is
    /// generated from its label and the node's UUID.
    pub fn queryable(&self, queryable: impl Into<String>) -> QueryableID {
        let label = queryable.into();

        QueryableID {
            uuid: Uuid::new_v3(&self.uuid, label.as_bytes()),
            label,
        }
    }
}

/// For internal use, this struct represents all the primitives
/// belonging to a node. It is used to create the layout of the node.
pub struct NodeDataLayout {
    pub inputs: HashSet<Uuid>,
    pub outputs: HashSet<Uuid>,
    pub queries: HashSet<Uuid>,
    pub queryables: HashSet<Uuid>,
}

/// For internal use, this struct represents the debug layout of a node.
/// It is used to store the labels of the primitives in the node.
pub struct NodeDebugLayout {
    pub labels: HashMap<Uuid, String>,
}

/// For internal use, this struct represents the layout of a node, both
/// the data and the debug layout.
pub struct NodeLayout {
    pub id: NodeID,
    pub data: NodeDataLayout,
    pub debug: NodeDebugLayout,
}

impl NodeLayout {
    /// Creates a new empty `NodeLayout` with the given `NodeID`.
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

    /// Adds a new input to the node layout. It returns a generic
    /// enum `PrimitiveID` that can be used to identify the input.
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

    /// Adds a new output to the node layout. It returns a generic
    /// enum `PrimitiveID` that can be used to identify the output.
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

    /// Adds a new query to the node layout. It returns a generic
    /// enum `PrimitiveID` that can be used to identify the query.
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

    /// Adds a new queryable to the node layout. It returns a generic
    /// enum `PrimitiveID` that can be used to identify the queryable.
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
