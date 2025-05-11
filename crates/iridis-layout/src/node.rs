use std::collections::{HashMap, HashSet};

use crate::prelude::*;

/// Represents a node
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeLayout {
    pub label: String,
    pub uuid: Uuid,
}

impl NodeLayout {
    /// Creates a new node layout with a unique UUID v4
    pub fn new(label: impl Into<String>) -> Self {
        NodeLayout {
            label: label.into(),
            uuid: Uuid::new_v4(),
        }
    }

    /// Creates a new input layout with a unique UUID v3 based on the node's UUID and input label
    pub fn input(&self, input: impl Into<String>) -> InputLayout {
        let label = input.into();
        InputLayout {
            uuid: Uuid::new_v3(&self.uuid, label.as_bytes()),
            label,
        }
    }

    /// Creates a new output layout with a unique UUID v3 based on the node's UUID and output label
    pub fn output(&self, output: impl Into<String>) -> OutputLayout {
        let label = output.into();
        OutputLayout {
            uuid: Uuid::new_v3(&self.uuid, label.as_bytes()),
            label,
        }
    }

    /// Creates a new query layout with a unique UUID v3 based on the node's UUID and query label
    pub fn query(&self, query: impl Into<String>) -> QueryLayout {
        let label = query.into();
        QueryLayout {
            uuid: Uuid::new_v3(&self.uuid, label.as_bytes()),
            label,
        }
    }

    /// Creates a new queryable layout with a unique UUID v3 based on the node's UUID and queryable label
    pub fn queryable(&self, queryable: impl Into<String>) -> QueryableLayout {
        let label = queryable.into();
        QueryableLayout {
            uuid: Uuid::new_v3(&self.uuid, label.as_bytes()),
            label,
        }
    }
}

/// This is the object passed to the user's lambda function to build the node's IO layout
pub struct Builder {
    /// The node layout this io builder is applied to
    pub layout: NodeLayout,

    /// The runtime only cares about input UUIDs
    pub inputs: HashSet<Uuid>,
    /// The runtime only cares about output UUIDs
    pub outputs: HashSet<Uuid>,
    /// The runtime only cares about query UUIDs
    pub queries: HashSet<Uuid>,
    /// The runtime only cares about queryable UUIDs
    pub queryables: HashSet<Uuid>,

    /// Labels for the node's IO, useful for debugging and visualization
    pub labels: HashMap<Uuid, String>,
}

impl Builder {
    pub fn new(layout: &NodeLayout) -> Self {
        Self {
            layout: layout.clone(),

            inputs: HashSet::new(),
            outputs: HashSet::new(),
            queryables: HashSet::new(),
            queries: HashSet::new(),

            labels: HashMap::new(),
        }
    }

    /// Creates a new input layout with the given label
    pub fn input(&mut self, input: impl Into<String>) -> IOLayout {
        let label: String = input.into();
        let layout = self.layout.input(&label);

        self.inputs.insert(layout.uuid);

        self.labels.insert(layout.uuid, label.clone());

        tracing::debug!(
            "Node '{}' (uuid: {}) added input layout with label: '{}' (uuid: {})",
            self.layout.label,
            self.layout.uuid,
            label,
            layout.uuid
        );

        layout.into()
    }

    /// Creates a new output layout with the given label
    pub fn output(&mut self, output: impl Into<String>) -> IOLayout {
        let label: String = output.into();
        let layout = self.layout.output(&label);

        self.outputs.insert(layout.uuid);

        self.labels.insert(layout.uuid, label.clone());

        tracing::debug!(
            "Node '{}' (uuid: {}) added output layout with label: '{}' (uuid: {})",
            self.layout.label,
            self.layout.uuid,
            label,
            layout.uuid
        );

        layout.into()
    }

    /// Creates a new query layout with the given label
    pub fn query(&mut self, query: impl Into<String>) -> IOLayout {
        let label: String = query.into();
        let layout = self.layout.query(&label);

        self.queries.insert(layout.uuid);

        self.labels.insert(layout.uuid, label.clone());

        tracing::debug!(
            "Node '{}' (uuid: {}) added query layout with label: '{}' (uuid: {})",
            self.layout.label,
            self.layout.uuid,
            label,
            layout.uuid
        );

        layout.into()
    }

    /// Creates a new queryable layout with the given label
    pub fn queryable(&mut self, queryable: impl Into<String>) -> IOLayout {
        let label: String = queryable.into();
        let layout = self.layout.queryable(&label);

        self.queryables.insert(layout.uuid);

        self.labels.insert(layout.uuid, label.clone());

        tracing::debug!(
            "Node '{}' (uuid: {}) added queryable layout with label: '{}' (uuid: {})",
            self.layout.label,
            self.layout.uuid,
            label,
            layout.uuid
        );

        layout.into()
    }
}
