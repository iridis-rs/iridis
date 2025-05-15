//! This module contains the necessary channels to communicate between
//! the different nodes in the dataflow.

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::prelude::{
    iridis_node::prelude::thirdparty::Uuid,
    thirdparty::tokio::{self, sync::Mutex},
    *,
};

type SharedMap<K, V> = Arc<Mutex<HashMap<K, V>>>;

/// This struct contains the channels used to communicate between
/// the different nodes in the dataflow.
pub struct RuntimeFlows {
    pub inputs_receivers: SharedMap<Uuid, MessageReceiver>,
    pub outputs_senders: SharedMap<Uuid, Vec<MessageSender>>,

    pub queries_senders: SharedMap<Uuid, MessageSender>, // other side is in 'queryables_receivers'
    pub queries_receivers: SharedMap<Uuid, MessageReceiver>, // other side is in 'queryables_senders'

    pub queryables_senders: SharedMap<Uuid, HashMap<Uuid, MessageSender>>, // receiver part in 'queries_receivers'
    pub queryables_receivers: SharedMap<Uuid, MessageReceiver>, // sender part in 'queries_senders'
}

impl RuntimeFlows {
    /// Creates a new `RuntimeFlows` struct from a `DataflowLayout`.
    pub fn new(layout: Arc<DataflowLayout>) -> Result<Self> {
        let mut inputs_receivers = HashMap::new();
        let mut outputs_senders = HashMap::new();

        let mut queries_senders = HashMap::new();
        let mut queries_receivers = HashMap::new();

        // Keep track of which queryables are connected to which queries
        let mut queryable_queries = HashMap::<Uuid, HashSet<Uuid>>::new();

        let mut queryables_senders = HashMap::new();
        let mut queryables_receivers = HashMap::new();

        for (a, b) in &layout.flows.connections {
            if layout.data.outputs.contains(a) && !inputs_receivers.contains_key(b) {
                let (output, input) = (a, b);

                let (sender, receiver) = tokio::sync::mpsc::channel(128);

                inputs_receivers.insert(*input, receiver);
                outputs_senders
                    .entry(*output)
                    .or_insert_with(Vec::new)
                    .push(sender);
            }

            if layout.data.queryables.contains(a) && !queries_receivers.contains_key(b) {
                let (queryable, query) = (a, b);

                let (sender, receiver) = tokio::sync::mpsc::channel(128);

                queries_receivers.insert(*query, receiver);

                let queryables_senders = queryables_senders
                    .entry(*queryable)
                    .or_insert_with(HashMap::new);

                if !queryables_senders.contains_key(query) {
                    queryables_senders.insert(*query, sender);
                }
            }

            if layout.data.queries.contains(a) && !queries_senders.contains_key(a) {
                let (query, queryable) = (a, b);

                if !queryables_receivers.contains_key(queryable) {
                    let (sender, receiver) = tokio::sync::mpsc::channel(128);

                    queries_senders.insert(*query, sender);
                    queryables_receivers.insert(*queryable, receiver);
                } else {
                    let other_query = match queryable_queries.get(queryable) {
                        Some(queries) => queries.iter().next(),
                        None => None,
                    }
                    .ok_or_eyre(report_primitive_not_found(*queryable))?;

                    let sender = queries_senders
                        .get(other_query)
                        .ok_or_eyre(report_primitive_not_found(*other_query))?
                        .clone();

                    queries_senders.insert(*query, sender);
                }

                queryable_queries
                    .entry(*queryable)
                    .or_default()
                    .insert(*query);
            }
        }

        Ok(Self {
            inputs_receivers: Arc::new(Mutex::new(inputs_receivers)),
            outputs_senders: Arc::new(Mutex::new(outputs_senders)),

            queries_senders: Arc::new(Mutex::new(queries_senders)),
            queries_receivers: Arc::new(Mutex::new(queries_receivers)),

            queryables_senders: Arc::new(Mutex::new(queryables_senders)),
            queryables_receivers: Arc::new(Mutex::new(queryables_receivers)),
        })
    }

    /// Extracts the `Inputs`, `Outputs`, `Queries` and `Queryables` for a given node
    pub fn node_primitives(
        &mut self,
        clock: Arc<HLC>,
        node: NodeID,
    ) -> (Inputs, Outputs, Queries, Queryables) {
        let inputs = Inputs::new(self.inputs_receivers.clone(), node.clone());
        let outputs = Outputs::new(self.outputs_senders.clone(), clock.clone(), node.clone());
        let queries = Queries::new(
            self.queries_senders.clone(),
            self.queries_receivers.clone(),
            clock.clone(),
            node.clone(),
        );
        let queryables = Queryables::new(
            self.queryables_senders.clone(),
            self.queryables_receivers.clone(),
            clock.clone(),
            node.clone(),
        );

        (inputs, outputs, queries, queryables)
    }
}
