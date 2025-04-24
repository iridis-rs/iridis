use crate::prelude::*;

/// Typed Input to receive data from the dataflow
#[derive(Debug)]
pub struct Input<T: ArrowMessage> {
    pub raw: RawInput,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: ArrowMessage> Input<T> {
    /// Create a new typed Input from a MessageReceiver, NodeLayout, and InputLayout
    pub fn new(rx: MessageReceiver, source: NodeLayout, layout: InputLayout) -> Self {
        Self {
            raw: RawInput::new(rx, source, layout),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Receive a message from the channel and converting it from Arrow format, blocking until one is available, don't use it
    /// in async context
    pub fn blocking_recv(&mut self) -> Result<(Header, T)> {
        let (header, data) = self
            .raw
            .blocking_recv()
            .wrap_err(report_error_receiving(&self.raw.source, &self.raw.layout))?;

        let message = T::try_from_arrow(data).wrap_err(
            report_failed_conversion_from_arrow::<T>(&self.raw.source, &self.raw.layout),
        )?;

        Ok((header, message))
    }

    /// Receive a message from the channel and converting it from Arrow format, asyncronously
    pub async fn recv(&mut self) -> Result<(Header, T)> {
        let (header, data) = self
            .raw
            .recv()
            .await
            .wrap_err(report_error_receiving(&self.raw.source, &self.raw.layout))?;

        let message = T::try_from_arrow(data).wrap_err(
            report_failed_conversion_from_arrow::<T>(&self.raw.source, &self.raw.layout),
        )?;

        Ok((header, message))
    }
}
