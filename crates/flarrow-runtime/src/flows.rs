use std::{collections::HashMap, sync::Arc};

use tokio::sync::{
    Mutex,
    mpsc::{Receiver, Sender},
};

use crate::prelude::*;

pub struct Flows {
    #[allow(clippy::type_complexity)]
    pub senders: Arc<Mutex<HashMap<OutputUUID, Vec<Sender<DataflowMessage>>>>>,
    #[allow(clippy::type_complexity)]
    pub receivers: Arc<Mutex<HashMap<InputUUID, Receiver<DataflowMessage>>>>,

    #[allow(clippy::type_complexity)]
    pub queries_senders: Arc<Mutex<HashMap<QueryUUID, Sender<DataflowMessage>>>>,
    #[allow(clippy::type_complexity)]
    pub queries_receivers: Arc<Mutex<HashMap<QueryUUID, Receiver<DataflowMessage>>>>,

    #[allow(clippy::type_complexity)]
    pub queryables_senders:
        Arc<Mutex<HashMap<QueryableUUID, HashMap<QueryUUID, Sender<DataflowMessage>>>>>,
    #[allow(clippy::type_complexity)]
    pub queryables_receivers: Arc<Mutex<HashMap<QueryableUUID, Receiver<DataflowMessage>>>>,
}

impl Flows {
    pub async fn new(
        layout: Arc<DataflowLayout>,
        flows: impl AsyncFnOnce(&mut Builder) -> Result<()>,
    ) -> Result<Self> {
        let mut connectors = Builder::new(layout)?;

        flows(&mut connectors).await?;

        Ok(Flows {
            senders: Arc::new(Mutex::new(connectors.senders)),
            receivers: Arc::new(Mutex::new(connectors.receivers)),
            queries_senders: Arc::new(Mutex::new(connectors.queries_senders)),
            queries_receivers: Arc::new(Mutex::new(connectors.queries_receivers)),
            queryables_senders: Arc::new(Mutex::new(connectors.queryables_senders)),
            queryables_receivers: Arc::new(Mutex::new(connectors.queryables_receivers)),
        })
    }
}

pub struct Builder {
    layout: Arc<DataflowLayout>,

    senders: HashMap<OutputUUID, Vec<Sender<DataflowMessage>>>,
    receivers: HashMap<InputUUID, Receiver<DataflowMessage>>,
    queries_senders: HashMap<QueryUUID, Sender<DataflowMessage>>,
    queries_receivers: HashMap<QueryUUID, Receiver<DataflowMessage>>,

    queryables_senders: HashMap<QueryableUUID, HashMap<QueryUUID, Sender<DataflowMessage>>>,
    queryables_receivers: HashMap<QueryableUUID, Receiver<DataflowMessage>>,
    tmp_queryables_receivers_send: HashMap<QueryableUUID, Sender<DataflowMessage>>,
}

impl Builder {
    pub fn new(layout: Arc<DataflowLayout>) -> Result<Self> {
        Ok(Self {
            layout,
            senders: HashMap::new(),
            receivers: HashMap::new(),
            queries_senders: HashMap::new(),
            queries_receivers: HashMap::new(),
            queryables_senders: HashMap::new(),
            queryables_receivers: HashMap::new(),
            tmp_queryables_receivers_send: HashMap::new(),
        })
    }

    pub fn service(
        &mut self,
        query: QueryUUID,
        queryable: QueryableUUID,
        capacity: Option<usize>,
    ) -> eyre::Result<&mut Self> {
        if !self.layout.queryables.contains(&queryable) {
            eyre::bail!("Queryable ID {} not found", queryable.0);
        }

        if !self.layout.queries.contains(&query) {
            eyre::bail!("Query ID {} not found", query.0);
        }

        if self.queries_receivers.contains_key(&query) {
            eyre::bail!("Query ID {} already mapped", query.0);
        }

        if self.queries_senders.contains_key(&query) {
            eyre::bail!("Query ID {} already mapped", query.0);
        }

        if let Some(senders) = self.queryables_senders.get(&queryable) {
            if senders.contains_key(&query) {
                eyre::bail!("Queryable ID {} already mapped", queryable.0);
            }
        }

        let (sender, receiver) = tokio::sync::mpsc::channel(capacity.unwrap_or(1024));

        self.queries_receivers.insert(query, receiver);
        self.queryables_senders
            .entry(queryable)
            .or_default()
            .insert(query, sender);

        if let Some(sender) = self.tmp_queryables_receivers_send.get(&queryable) {
            self.queries_senders.insert(query, sender.clone());
        } else {
            let (sender, receiver) = tokio::sync::mpsc::channel(capacity.unwrap_or(1024));

            self.tmp_queryables_receivers_send
                .insert(queryable, sender.clone());
            self.queries_senders.insert(query, sender);
            self.queryables_receivers.insert(queryable, receiver);
        }

        Ok(self)
    }

    pub fn connect(
        &mut self,
        input: InputUUID,
        output: OutputUUID,
        capacity: Option<usize>,
    ) -> eyre::Result<&mut Self> {
        if !self.layout.inputs.contains(&input) {
            eyre::bail!("Input ID {} not found", input.0);
        }

        if !self.layout.outputs.contains(&output) {
            eyre::bail!("Output ID {} not found", output.0);
        }

        if self.receivers.contains_key(&input) {
            eyre::bail!("Input ID {} already mapped", input.0);
        }

        let (sender, receiver) = tokio::sync::mpsc::channel(capacity.unwrap_or(1024));

        self.senders.entry(output).or_default().push(sender);
        self.receivers.insert(input, receiver);

        Ok(self)
    }
}
