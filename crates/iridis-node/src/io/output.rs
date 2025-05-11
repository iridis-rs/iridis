use std::sync::Arc;

use crate::prelude::*;
use thirdparty::arrow_array::Array;

/// Typed Output to receive data from the dataflow
pub struct Output<T: ArrowMessage> {
    pub raw: RawOutput,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: ArrowMessage> Output<T> {
    /// Create a new typed Output from a MessageSender, NodeLayout, and OutputLayout
    pub fn new(
        tx: Vec<MessageSender>,
        clock: Arc<HLC>,
        source: NodeLayout,
        layout: OutputLayout,
    ) -> Self {
        Self {
            raw: RawOutput::new(tx, clock, source, layout),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Send a message to the output asynchronously.
    pub async fn send(&self, data: T) -> Result<()> {
        self.raw
            .send(
                data.try_into_arrow()
                    .wrap_err(report_failed_conversion_to_arrow::<T>(
                        &self.raw.source,
                        &self.raw.layout,
                    ))?
                    .into_data(),
            )
            .await
    }
}
