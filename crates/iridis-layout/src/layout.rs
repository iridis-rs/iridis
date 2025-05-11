use std::{
    collections::{HashMap, HashSet},
    fmt,
    sync::Arc,
};

use crate::prelude::*;

/// Represents a flattened dataflow layout
pub struct DataflowLayout {
    /// Inputs present in the dataflow
    pub inputs: HashSet<Uuid>,
    /// Outputs present in the dataflow
    pub outputs: HashSet<Uuid>,
    /// Queries present in the dataflow
    pub queries: HashSet<Uuid>,
    /// Queryables present in the dataflow
    pub queryables: HashSet<Uuid>,

    /// Labels for nodes and IO, useful for debugging and visualization
    pub labels: HashMap<Uuid, String>,
    /// Nodes representation with their IO, useful for debugging and visualization
    pub nodes: HashMap<Uuid, HashSet<Uuid>>,
}

impl Default for DataflowLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl DataflowLayout {
    /// Creates a new empty dataflow layout
    pub fn new() -> Self {
        Self {
            inputs: HashSet::new(),
            outputs: HashSet::new(),
            queryables: HashSet::new(),
            queries: HashSet::new(),

            labels: HashMap::new(),
            nodes: HashMap::new(),
        }
    }

    /// Adds a new node to the dataflow layout, providing its label and a builder function for its IO.
    /// For convenience, the node layout is returned, together with the result of the builder function.
    pub async fn node<T>(
        &mut self,
        label: impl Into<String>,
        builder_function: impl AsyncFnOnce(&mut Builder) -> T,
    ) -> (NodeLayout, T) {
        let label = label.into();
        let layout = NodeLayout::new(&label);
        let mut io = Builder::new(&layout);

        let result = builder_function(&mut io).await;

        self.nodes.insert(
            layout.uuid,
            io.inputs
                .union(&io.outputs)
                .chain(io.queries.union(&io.queryables))
                .cloned()
                .collect(),
        );

        self.inputs.extend(io.inputs);
        self.outputs.extend(io.outputs);
        self.queries.extend(io.queries);
        self.queryables.extend(io.queryables);
        self.labels.extend(io.labels);

        self.labels.insert(layout.uuid, label.clone());

        tracing::debug!("Node '{}' (uuid: {}) created", label, layout.uuid);

        (layout, result)
    }

    /// Access the label of an entity in the dataflow layout (node, input, output, query, queryable)
    pub fn label(&self, uuid: impl Into<Uuid>) -> String {
        let uuid = uuid.into();

        self.labels.get(&uuid).cloned().unwrap_or_default()
    }

    /// Build the dataflow layout by making it immutable and returning an Arc
    pub fn build(self) -> Arc<Self> {
        Arc::new(self)
    }
}

impl fmt::Debug for DataflowLayout {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        #[derive(Debug)]
        struct Layout {
            #[allow(dead_code)]
            id: (String, Uuid),

            inputs: HashSet<(String, Uuid)>,
            outputs: HashSet<(String, Uuid)>,
            queryables: HashSet<(String, Uuid)>,
            queries: HashSet<(String, Uuid)>,
        }

        let mut nodes = Vec::new();

        for (&node, io) in &self.nodes {
            let mut layout = Layout {
                id: (self.label(node), node),

                inputs: HashSet::new(),
                outputs: HashSet::new(),
                queryables: HashSet::new(),
                queries: HashSet::new(),
            };

            for &io in io {
                if self.inputs.contains(&io) {
                    layout.inputs.insert((self.label(io), io));
                }
                if self.outputs.contains(&io) {
                    layout.outputs.insert((self.label(io), io));
                }
                if self.queryables.contains(&io) {
                    layout.queryables.insert((self.label(io), io));
                }
                if self.queries.contains(&io) {
                    layout.queries.insert((self.label(io), io));
                }
            }

            nodes.push(layout);
        }

        writeln!(f, "{:#?}", nodes)
    }
}
