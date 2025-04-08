use std::{collections::HashMap, sync::Arc};

use tokio::sync::{
    Mutex,
    broadcast::{Receiver, Sender},
};

use crate::prelude::*;

pub struct Flows {
    pub senders: Arc<Mutex<HashMap<OutputID, Sender<DataflowMessage>>>>,
    pub receivers: Arc<Mutex<HashMap<InputID, Receiver<DataflowMessage>>>>,
}

impl Flows {
    pub async fn new(
        layout: Arc<DataflowLayout>,
        flows: impl AsyncFn(&mut Connector) -> Result<()>,
    ) -> Result<Self> {
        let mut connectors = Connector::new(layout)?;

        flows(&mut connectors).await?;

        Ok(Flows {
            senders: Arc::new(Mutex::new(connectors.senders)),
            receivers: Arc::new(Mutex::new(connectors.receivers)),
        })
    }
}

pub struct Connector {
    layout: Arc<DataflowLayout>,

    senders: HashMap<OutputID, Sender<DataflowMessage>>,
    receivers: HashMap<InputID, Receiver<DataflowMessage>>,
}

impl Connector {
    pub fn new(layout: Arc<DataflowLayout>) -> Result<Self> {
        Ok(Self {
            layout,
            senders: HashMap::new(),
            receivers: HashMap::new(),
        })
    }

    pub fn connect(&mut self, input: InputID, output: OutputID) -> eyre::Result<&mut Self> {
        if !self.layout.inputs.contains(&input) {
            eyre::bail!("Input ID {} not found", input.0);
        }

        if !self.layout.outputs.contains(&output) {
            eyre::bail!("Output ID {} not found", output.0);
        }

        if self.receivers.contains_key(&input) {
            eyre::bail!("Input ID {} already mapped", input.0);
        }

        let receiver = match self.senders.get(&output) {
            Some(sender) => sender.subscribe(),
            None => {
                let (sender, receiver) = tokio::sync::broadcast::channel(1024);
                self.senders.insert(output, sender);
                receiver
            }
        };

        self.receivers.insert(input, receiver);

        Ok(self)
    }
}
