//! This module defines the complete layout of a `dataflow` application.

use std::{
    collections::{HashMap, HashSet},
    fmt,
    sync::Arc,
};

use crate::prelude::{thirdparty::tokio::sync::Mutex, *};

/// Represents the data layout of the application.
#[derive(Debug, Clone)]
pub struct DataLayout {
    pub inputs: HashSet<Uuid>,
    pub outputs: HashSet<Uuid>,
    pub queries: HashSet<Uuid>,
    pub queryables: HashSet<Uuid>,
}

/// Represents the debug layout of the application: the labels
/// and the nodes/primitives relationship.
#[derive(Debug, Clone)]
pub struct DebugLayout {
    pub labels: HashMap<Uuid, String>,
    pub nodes: HashMap<Uuid, HashSet<Uuid>>,
}

impl DebugLayout {
    /// Gets the label of a primitive or a Node by its UUID.
    pub fn label(&self, uuid: impl AsRef<Uuid>) -> String {
        self.labels.get(uuid.as_ref()).cloned().unwrap_or_default()
    }
}

/// Represents the a `Dataflow` application! This is the main
/// struct that contains all the data and debug layouts.
#[derive(Clone)]
pub struct DataflowLayout {
    pub data: DataLayout,
    pub debug: DebugLayout,
    pub flows: FlowLayout,
}

/// Represents a shared `Data` only layout. It is used to construct
/// a `DataflowLayout` easily.
#[derive(Debug, Clone)]
pub struct SharedDataLayout {
    pub data: Arc<Mutex<DataLayout>>,
    pub debug: Arc<Mutex<DebugLayout>>,
}

impl DataflowLayout {
    /// Creates a new empty `SharedDataLayout` that must be `finished` to create
    /// the final `DataflowLayout`.
    pub fn empty() -> SharedDataLayout {
        SharedDataLayout {
            data: Arc::new(Mutex::new(DataLayout {
                inputs: HashSet::new(),
                outputs: HashSet::new(),
                queryables: HashSet::new(),
                queries: HashSet::new(),
            })),
            debug: Arc::new(Mutex::new(DebugLayout {
                labels: HashMap::new(),
                nodes: HashMap::new(),
            })),
        }
    }

    /// Gets the label of a primitive or a Node by its UUID.
    pub fn label(&self, uuid: impl AsRef<Uuid>) -> String {
        self.debug.label(uuid)
    }
}

impl SharedDataLayout {
    /// Creates a new `Node` with the given label. Provide an `async` closure
    /// to add primitives to the node (such as Inputs, Queries etc...)
    pub async fn node<T>(
        &self,
        label: impl Into<String>,
        layout_builder: impl AsyncFnOnce(&mut NodeLayout) -> T,
    ) -> (NodeID, T) {
        let label = label.into();
        let id = NodeID::new(&label);
        let mut layout = NodeLayout::new(&id);

        let result = layout_builder(&mut layout).await;

        let mut debug = self.debug.lock().await;

        debug.nodes.insert(
            id.uuid,
            layout
                .data
                .inputs
                .union(&layout.data.outputs)
                .chain(layout.data.queries.union(&layout.data.queryables))
                .cloned()
                .collect(),
        );

        let mut data = self.data.lock().await;

        data.inputs.extend(layout.data.inputs);
        data.outputs.extend(layout.data.outputs);
        data.queries.extend(layout.data.queries);
        data.queryables.extend(layout.data.queryables);

        debug.labels.extend(layout.debug.labels);
        debug.labels.insert(id.uuid, label.clone());

        tracing::debug!("Node '{}' (uuid: {}) created", label, id.uuid);

        (id, result)
    }

    /// Creates a `DataflowLayout` from the current `SharedDataLayout` and the given
    /// `flows` function. The `flows` function is an `async` closure that takes a
    /// `FlowLayout` that can be used to connect the primitives together.
    pub async fn finish(
        self,
        flows: impl AsyncFnOnce(&mut FlowLayout) -> Result<()>,
    ) -> Result<Arc<DataflowLayout>> {
        let mut layout = FlowLayout {
            connections: HashSet::new(),
        };

        flows(&mut layout).await.wrap_err("Failed to build flows")?;

        let data = self.data.lock().await.clone();
        let debug = self.debug.lock().await.clone();

        for (a, b) in &layout.connections {
            match (data.outputs.contains(a), data.inputs.contains(b)) {
                (true, true) => {}
                (false, false) => match (data.queries.contains(a), data.queryables.contains(b)) {
                    (true, true) => {}
                    (false, false) => match (data.queryables.contains(a), data.queries.contains(b))
                    {
                        (true, true) => {}
                        _ => {
                            eyre::bail!(
                                "Invalid connection between '{}' and '{}'",
                                debug.label(a),
                                debug.label(b)
                            );
                        }
                    },
                    _ => {
                        eyre::bail!(
                            "Invalid connection between '{}' and '{}'",
                            debug.label(a),
                            debug.label(b)
                        );
                    }
                },
                _ => {
                    eyre::bail!(
                        "Invalid connection between '{}' and '{}'",
                        debug.label(a),
                        debug.label(b)
                    );
                }
            }
        }

        Ok(Arc::new(DataflowLayout {
            data,
            debug,
            flows: layout,
        }))
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

        for (&node, io) in &self.debug.nodes {
            let mut layout = Layout {
                id: (self.label(node), node),

                inputs: HashSet::new(),
                outputs: HashSet::new(),
                queryables: HashSet::new(),
                queries: HashSet::new(),
            };

            for &io in io {
                if self.data.inputs.contains(&io) {
                    layout.inputs.insert((self.label(io), io));
                }
                if self.data.outputs.contains(&io) {
                    layout.outputs.insert((self.label(io), io));
                }
                if self.data.queryables.contains(&io) {
                    layout.queryables.insert((self.label(io), io));
                }
                if self.data.queries.contains(&io) {
                    layout.queries.insert((self.label(io), io));
                }
            }

            nodes.push(layout);
        }

        writeln!(f, "{:#?}", nodes)
    }
}
