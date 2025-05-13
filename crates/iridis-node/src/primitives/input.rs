use crate::prelude::*;

/// Typed Input to receive data from the dataflow
#[derive(Debug)]
pub struct Input<T: ArrowMessage> {
    pub raw: RawInput,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: ArrowMessage> Input<T> {
    /// Create a new typed Input from a MessageReceiver, NodeID, and InputID
    pub fn new(rx: MessageReceiver, source: NodeID, layout: InputID) -> Self {
        Self {
            raw: RawInput::new(rx, source, layout),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Receive a message from the channel and converting it from Arrow format, asyncronously
    pub async fn recv(&mut self) -> Result<TypedDataflowMessage<T>> {
        self.raw
            .recv()
            .await?
            .try_into()
            .wrap_err(report_failed_conversion_from_arrow::<T>(
                &self.raw.source,
                &self.raw.layout,
            ))
    }
}
