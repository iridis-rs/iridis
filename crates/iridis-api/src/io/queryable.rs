use std::{collections::HashMap, sync::Arc};

use crate::prelude::*;
use thirdparty::arrow_array::Array;

/// Typed Queryable to queryable data to the dataflow
pub struct Queryable<T: ArrowMessage, F: ArrowMessage> {
    pub raw: RawQueryable,

    _phantom: std::marker::PhantomData<(T, F)>,
}

impl<T: ArrowMessage, F: ArrowMessage> Queryable<T, F> {
    /// Create a new typed Queryable
    pub fn new(
        tx: HashMap<Uuid, MessageSender>,
        rx: MessageReceiver,
        clock: Arc<HLC>,
        source: NodeLayout,
        layout: QueryableLayout,
    ) -> Self {
        Self {
            raw: RawQueryable::new(tx, rx, clock, source, layout),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Let the queryable handle a message, converting it from Arrow format, asynchrously
    pub async fn on_demand(&mut self, response: impl AsyncFnOnce(T) -> Result<F>) -> Result<()> {
        let source = self.raw.source.clone();
        let layout = self.raw.layout.clone();

        self.raw
            .on_demand(async move |message| {
                let result = response(
                    T::try_from_arrow(message.data)
                        .wrap_err(report_failed_conversion_from_arrow::<T>(&source, &layout))?,
                )
                .await?;

                Ok(result
                    .try_into_arrow()
                    .wrap_err(report_failed_conversion_to_arrow::<F>(&source, &layout))?
                    .into_data())
            })
            .await
    }
}
