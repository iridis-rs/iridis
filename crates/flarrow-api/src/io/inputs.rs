use eyre::{Context, OptionExt};

use arrow_data::ArrayData;

use crate::prelude::*;

pub struct RawInput {
    pub rx: DataflowReceiver,
}

impl RawInput {
    pub fn new(rx: DataflowReceiver) -> Self {
        Self { rx }
    }

    pub fn recv(&mut self) -> Result<(Header, ArrayData)> {
        let DataflowMessage { header, data } = self
            .rx
            .blocking_recv()
            .ok_or_eyre("Failed to receive from this input")?;

        Ok((header, data))
    }

    pub async fn recv_async(&mut self) -> Result<(Header, ArrayData)> {
        let DataflowMessage { header, data } = self
            .rx
            .recv()
            .await
            .ok_or_eyre("Failed to receive from this input")?;

        Ok((header, data))
    }
}

pub struct Input<T: ArrowMessage> {
    pub raw: RawInput,

    _phantom: std::marker::PhantomData<T>,
}

impl<T: ArrowMessage> Input<T> {
    pub fn new(rx: DataflowReceiver) -> Self {
        Self {
            raw: RawInput::new(rx),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn recv(&mut self) -> Result<(Header, T)> {
        let (header, data) = self.raw.recv()?;

        Ok((
            header,
            T::try_from_arrow(data).wrap_err("Failed to convert arrow 'data' to message T")?,
        ))
    }

    pub async fn recv_async(&mut self) -> Result<(Header, T)> {
        let (header, data) = self.raw.recv_async().await?;

        Ok((
            header,
            T::try_from_arrow(data).wrap_err("Failed to convert arrow 'data' to message T")?,
        ))
    }
}

pub struct Inputs {
    node: NodeUUID,

    receivers: SharedMap<InputUUID, DataflowReceiver>,
}

impl Inputs {
    pub fn new(node: NodeUUID, receivers: SharedMap<InputUUID, DataflowReceiver>) -> Self {
        Self { node, receivers }
    }

    pub async fn raw(&mut self, input: impl Into<String>) -> Result<RawInput> {
        let id = self.node.input(input);

        let receiver = self
            .receivers
            .lock()
            .await
            .remove(&id)
            .ok_or_eyre(format!("Input {} not found", id.0))?;

        Ok(RawInput::new(receiver))
    }

    pub async fn with<T: ArrowMessage>(&mut self, input: impl Into<String>) -> Result<Input<T>> {
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
