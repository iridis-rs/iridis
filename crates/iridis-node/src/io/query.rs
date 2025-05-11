use std::sync::Arc;

use crate::prelude::*;
use thirdparty::arrow_array::Array;

/// Typed Query to query data from the dataflow
pub struct Query<T: ArrowMessage, F: ArrowMessage> {
    pub raw: RawQuery,

    _phantom: std::marker::PhantomData<(T, F)>,
}

impl<T: ArrowMessage, F: ArrowMessage> Query<T, F> {
    /// Create a new typed Query
    pub fn new(
        tx: MessageSender,
        rx: MessageReceiver,
        clock: Arc<HLC>,
        source: NodeLayout,
        layout: QueryLayout,
    ) -> Self {
        Self {
            raw: RawQuery::new(tx, rx, clock, source, layout),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Query a message from the channel and converting it from Arrow format, asynchronously
    pub async fn query(&mut self, data: T) -> Result<(Header, F)> {
        let DataflowMessage { header, data } = self
            .raw
            .query(
                data.try_into_arrow()
                    .wrap_err(report_failed_conversion_to_arrow::<T>(
                        &self.raw.source,
                        &self.raw.layout,
                    ))?
                    .into_data(),
            )
            .await?;

        Ok((
            header,
            F::try_from_arrow(data).wrap_err(report_failed_conversion_from_arrow::<F>(
                &self.raw.source,
                &self.raw.layout,
            ))?,
        ))
    }
}
