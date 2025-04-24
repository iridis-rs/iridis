use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;
use uhlc::HLC;
use uuid::Uuid;

use crate::prelude::*;

type SharedMap<K, V> = Arc<Mutex<HashMap<K, V>>>;

/// Represents the collection of flows in the system (all the MPSC channels)
#[derive(Debug, Clone)]
pub struct Flows {
    pub inputs_receivers: SharedMap<Uuid, MessageReceiver>,
    pub outputs_senders: SharedMap<Uuid, Vec<MessageSender>>,

    pub queries_senders: SharedMap<Uuid, MessageSender>, // other side is in 'queryables_receivers'
    pub queries_receivers: SharedMap<Uuid, MessageReceiver>, // other side is in 'queryables_senders'

    pub queryables_senders: SharedMap<Uuid, HashMap<Uuid, MessageSender>>, // receiver part in 'queries_receivers'
    pub queryables_receivers: SharedMap<Uuid, MessageReceiver>, // sender part in 'queries_senders'
}

/// This is the builder struct available for the user closure when creating flows.
pub struct FlowsBuilder {
    pub layout: Arc<DataflowLayout>,

    pub inputs_receivers: HashMap<Uuid, MessageReceiver>,
    pub outputs_senders: HashMap<Uuid, Vec<MessageSender>>,

    pub queries_senders: HashMap<Uuid, MessageSender>,
    pub queries_receivers: HashMap<Uuid, MessageReceiver>,

    pub queryables_senders: HashMap<Uuid, HashMap<Uuid, MessageSender>>,
    pub queryables_receivers: HashMap<Uuid, MessageReceiver>,
}

impl Flows {
    /// Creates a new Flows instance with a Building async closure
    pub async fn new(
        layout: Arc<DataflowLayout>,
        flows: impl AsyncFnOnce(&mut FlowsBuilder) -> Result<()>,
    ) -> Result<Self> {
        let mut builder = FlowsBuilder::new(layout);

        flows(&mut builder).await?;

        Ok(Flows {
            outputs_senders: Arc::new(Mutex::new(builder.outputs_senders)),
            inputs_receivers: Arc::new(Mutex::new(builder.inputs_receivers)),
            queries_senders: Arc::new(Mutex::new(builder.queries_senders)),
            queries_receivers: Arc::new(Mutex::new(builder.queries_receivers)),
            queryables_senders: Arc::new(Mutex::new(builder.queryables_senders)),
            queryables_receivers: Arc::new(Mutex::new(builder.queryables_receivers)),
        })
    }

    /// This is intended to be used only by the Runtime when loading nodes
    pub fn node_io(
        &self,
        clock: Arc<HLC>,
        source: NodeLayout,
    ) -> (Inputs, Outputs, Queries, Queryables) {
        let inputs = Inputs::new(self.inputs_receivers.clone(), source.clone());
        let outputs = Outputs::new(self.outputs_senders.clone(), clock.clone(), source.clone());
        let queries = Queries::new(
            self.queries_senders.clone(),
            self.queries_receivers.clone(),
            clock.clone(),
            source.clone(),
        );
        let queryables = Queryables::new(
            self.queryables_senders.clone(),
            self.queryables_receivers.clone(),
            clock.clone(),
            source.clone(),
        );

        (inputs, outputs, queries, queryables)
    }
}

impl FlowsBuilder {
    pub fn new(layout: Arc<DataflowLayout>) -> Self {
        Self {
            layout,
            inputs_receivers: HashMap::new(),
            outputs_senders: HashMap::new(),
            queries_senders: HashMap::new(),
            queries_receivers: HashMap::new(),
            queryables_senders: HashMap::new(),
            queryables_receivers: HashMap::new(),
        }
    }

    /// Connect input to output (Output --> Input)
    fn connect_input_output(&mut self, input: Uuid, output: Uuid, capacity: usize) -> Result<()> {
        if !self.layout.inputs.contains(&input) {
            eyre::bail!(
                "Input '{}' (uuid: {}) not found in the dataflow layout created",
                self.layout.label(input),
                input
            );
        }

        if !self.layout.outputs.contains(&output) {
            eyre::bail!(
                "Output '{}' (uuid: {}) not found in the dataflow layout created",
                self.layout.label(output),
                output
            );
        }

        if self.inputs_receivers.contains_key(&input) {
            eyre::bail!(
                "Input '{}' (uuid: {}) already mapped",
                self.layout.label(input),
                input
            );
        }

        let (sender, receiver) = tokio::sync::mpsc::channel(capacity);

        self.outputs_senders.entry(output).or_default().push(sender);
        self.inputs_receivers.insert(input, receiver);

        tracing::debug!(
            "Connecting input '{}' (uuid: {}) to output '{}' (uuid: {})",
            self.layout.label(input),
            input,
            self.layout.label(output),
            output
        );

        Ok(())
    }

    /// Connect query to queryable (Queryable --> Query)
    fn connect_query_queryable(
        &mut self,
        query: Uuid,
        queryable: Uuid,
        capacity: usize,
    ) -> Result<()> {
        if !self.layout.queryables.contains(&queryable) || !self.layout.queries.contains(&query) {
            eyre::bail!(
                "Queryable '{}' (uuid: {}) not found in the dataflow layout created",
                self.layout.label(queryable),
                queryable
            );
        }

        if !self.layout.queries.contains(&query) {
            eyre::bail!(
                "Query '{}' (uuid: {}) not found in the dataflow layout created",
                self.layout.label(query),
                query
            );
        }

        if self.queries_senders.contains_key(&query) || self.queries_receivers.contains_key(&query)
        {
            eyre::bail!(
                "Query '{}' (uuid: {}) is already connected",
                self.layout.label(query),
                query
            );
        }

        if let Some(senders) = self.queryables_senders.get(&queryable) {
            if senders.contains_key(&query) {
                eyre::bail!(
                    "Queryable '{}' (uuid: {}) already mapped",
                    self.layout.label(queryable),
                    queryable
                );
            }
        }

        if let std::collections::hash_map::Entry::Vacant(e) =
            self.queryables_receivers.entry(queryable)
        {
            let (sender, receiver) = tokio::sync::mpsc::channel(capacity);

            e.insert(receiver);
            self.queries_senders.insert(query, sender);
        } else {
            let other_query = *match self.queryables_senders.get(&queryable) {
                Some(senders) => senders.keys().next(),
                None => None,
            }
            .ok_or_eyre(format!(
                "Queryable '{}' (uuid: {}) is not well connected. This is a big error please report it",
                self.layout.label(queryable),
                queryable,
            ))?;

            let sender = self.queries_senders.get(&other_query).ok_or_eyre(format!(
                "Query '{}' (uuid: {}) is not well connected. This is a big error please report it",
                self.layout.label(other_query),
                other_query,
            ))?.clone();

            self.queries_senders.insert(query, sender);
        }

        let (sender, receiver) = tokio::sync::mpsc::channel(capacity);
        self.queries_receivers.insert(query, receiver);
        self.queryables_senders
            .entry(queryable)
            .or_default()
            .insert(query, sender);

        tracing::debug!(
            "Connecting query '{}' (uuid: {}) to queryable '{}' (uuid: {})",
            self.layout.label(query),
            query,
            self.layout.label(queryable),
            queryable
        );

        Ok(())
    }

    /// Connect two IO (a -> b or b -> a).
    /// Acceptable format (a, b) in {Input, Output}
    /// or (a, b) in {Query, Queryable} with a != b
    pub fn connect(&mut self, a: IOLayout, b: IOLayout, capacity: Option<usize>) -> Result<()> {
        match (a, b) {
            (IOLayout::Input(input), IOLayout::Output(output)) => {
                self.connect_input_output(input.uuid, output.uuid, capacity.unwrap_or(128))
            }
            (IOLayout::Query(query), IOLayout::Queryable(queryable)) => {
                self.connect_query_queryable(query.uuid, queryable.uuid, capacity.unwrap_or(128))
            }
            (IOLayout::Output(output), IOLayout::Input(input)) => {
                self.connect_input_output(input.uuid, output.uuid, capacity.unwrap_or(128))
            }
            (IOLayout::Queryable(queryable), IOLayout::Query(query)) => {
                self.connect_query_queryable(query.uuid, queryable.uuid, capacity.unwrap_or(128))
            }
            _ => Err(eyre::eyre!(
                "Invalid connection! types `a` and `b` must verify: a != b and (a, b) in {{Input, Output}} or {{Query, Queryable}}"
            )),
        }
    }
}
