use eyre::{Context, OptionExt};
use std::{collections::HashMap, sync::Arc};

use tokio::sync::{
    Mutex,
    broadcast::{Receiver, Sender},
};

use arrow_array::Array;
use arrow_data::ArrayData;

use crate::prelude::*;

pub struct RawOutput {
    clock: Arc<uhlc::HLC>,

    pub tx: Sender<DataflowMessage>,
}

impl RawOutput {
    pub fn new(clock: Arc<uhlc::HLC>, tx: Sender<DataflowMessage>) -> Self {
        Self { clock, tx }
    }

    pub fn send(&self, data: ArrayData) -> eyre::Result<()> {
        let data = DataflowMessage {
            header: Header {
                timestamp: self.clock.new_timestamp(),
            },
            data,
        };

        self.tx
            .send(data)
            .map(|_| ())
            .map_err(eyre::Report::msg)
            .wrap_err("Failed to send the message")
    }
}

pub struct Output<T: ArrowMessage> {
    pub raw: RawOutput,

    _phantom: std::marker::PhantomData<T>,
}

impl<T: ArrowMessage> Output<T> {
    pub fn new(clock: Arc<uhlc::HLC>, tx: Sender<DataflowMessage>) -> Self {
        Self {
            raw: RawOutput::new(clock, tx),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn send(&self, data: T) -> eyre::Result<()> {
        self.raw.send(
            data.try_into_arrow()
                .wrap_err("Failed to convert arrow 'data' to message T")?
                .into_data(),
        )
    }
}

pub struct RawInput {
    pub rx: Receiver<DataflowMessage>,
}

impl RawInput {
    pub fn new(rx: Receiver<DataflowMessage>) -> Self {
        Self { rx }
    }

    pub fn recv(&mut self) -> eyre::Result<(Header, ArrayData)> {
        let DataflowMessage { header, data } = self
            .rx
            .blocking_recv()
            .map_err(eyre::Report::msg)
            .wrap_err("Failed to receive from this input")?;

        Ok((header, data))
    }

    pub async fn recv_async(&mut self) -> eyre::Result<(Header, ArrayData)> {
        let DataflowMessage { header, data } = self
            .rx
            .recv()
            .await
            .map_err(eyre::Report::msg)
            .wrap_err("Failed to receive from this input")?;

        Ok((header, data))
    }
}

pub struct Input<T: ArrowMessage> {
    pub raw: RawInput,

    _phantom: std::marker::PhantomData<T>,
}

impl<T: ArrowMessage> Input<T> {
    pub fn new(rx: Receiver<DataflowMessage>) -> Self {
        Self {
            raw: RawInput::new(rx),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn recv(&mut self) -> eyre::Result<(Header, T)> {
        let (header, data) = self.raw.recv()?;

        Ok((
            header,
            T::try_from_arrow(data).wrap_err("Failed to convert arrow 'data' to message T")?,
        ))
    }

    pub async fn recv_async(&mut self) -> eyre::Result<(Header, T)> {
        let (header, data) = self.raw.recv_async().await?;

        Ok((
            header,
            T::try_from_arrow(data).wrap_err("Failed to convert arrow 'data' to message T")?,
        ))
    }
}

pub struct Inputs {
    node: NodeID,
    receivers: Arc<Mutex<HashMap<InputID, Receiver<DataflowMessage>>>>,
}

impl Inputs {
    pub fn new(
        node: NodeID,
        receivers: Arc<Mutex<HashMap<InputID, Receiver<DataflowMessage>>>>,
    ) -> Self {
        Self { node, receivers }
    }

    pub async fn raw(&mut self, input: impl Into<String>) -> eyre::Result<RawInput> {
        let id = self.node.input(input);

        let receiver = self
            .receivers
            .lock()
            .await
            .remove(&id)
            .ok_or_eyre(format!("Input {} not found", id.0))?;

        Ok(RawInput::new(receiver))
    }

    pub async fn with<T: ArrowMessage>(
        &mut self,
        input: impl Into<String>,
    ) -> eyre::Result<Input<T>> {
        let id = self.node.input(input);

        let receiver = self
            .receivers
            .lock()
            .await
            .remove(&id)
            .ok_or_eyre(format!("Input {} not found", id.0))?;

        Ok(Input::new(receiver))
    }
}

pub struct Outputs {
    node: NodeID,
    clock: Arc<uhlc::HLC>,
    senders: Arc<Mutex<HashMap<OutputID, Sender<DataflowMessage>>>>,
}

impl Outputs {
    pub fn new(
        node: NodeID,
        clock: Arc<uhlc::HLC>,
        senders: Arc<Mutex<HashMap<OutputID, Sender<DataflowMessage>>>>,
    ) -> Self {
        Self {
            node,
            clock,
            senders,
        }
    }

    pub async fn raw(&mut self, output: impl Into<String>) -> eyre::Result<RawOutput> {
        let id = self.node.output(output);

        let sender = self
            .senders
            .lock()
            .await
            .remove(&id)
            .ok_or_eyre(format!("Output {} not found", id.0))?;

        Ok(RawOutput::new(self.clock.clone(), sender))
    }

    pub async fn with<T: ArrowMessage>(
        &mut self,
        output: impl Into<String>,
    ) -> eyre::Result<Output<T>> {
        let id = self.node.output(output);

        let sender = self
            .senders
            .lock()
            .await
            .remove(&id)
            .ok_or_eyre(format!("Output {} not found", id.0))?;

        Ok(Output::new(self.clock.clone(), sender))
    }
}
