use std::{collections::HashMap, sync::Arc};

use crate::prelude::*;
use thirdparty::tokio::sync::Mutex;

type SharedMap<K, V> = Arc<Mutex<HashMap<K, V>>>;
type Senders = SharedMap<Uuid, HashMap<Uuid, MessageSender>>;
type Receivers = SharedMap<Uuid, MessageReceiver>;

/// Queryables let you manage queryable connections during a node *implementation*
pub struct Queryables {
    senders: Senders,
    receivers: Receivers,

    clock: Arc<uhlc::HLC>,

    source: NodeID,
}

impl Queryables {
    /// Creates a new instance of 'Queryables'
    pub fn new(
        senders: Senders,
        receivers: Receivers,
        clock: Arc<uhlc::HLC>,
        source: NodeID,
    ) -> Self {
        Self {
            clock,
            senders,
            receivers,
            source,
        }
    }

    async fn compute(
        &mut self,
        queryable: impl Into<String>,
    ) -> Result<(HashMap<Uuid, MessageSender>, MessageReceiver, QueryableID)> {
        let label: String = queryable.into();
        let layout = self.source.queryable(&label);

        let senders = self
            .senders
            .lock()
            .await
            .remove(&layout.uuid)
            .ok_or_eyre(report_io_not_found(&self.source, &layout))?;

        let receiver = self
            .receivers
            .lock()
            .await
            .remove(&layout.uuid)
            .ok_or_eyre(report_io_not_found(&self.source, &layout))?;

        Ok((senders, receiver, layout))
    }

    /// Creates a new raw Queryable, this raw queryable has no type information so you have
    /// to manually transform it
    pub async fn raw(&mut self, queryable: impl Into<String>) -> Result<RawQueryable> {
        let (senders, receivers, layout) = self.compute(queryable).await?;

        tracing::debug!(
            "Creating new raw queryable '{}' (uuid: {}) for node '{}' (uuid: {})",
            layout.label,
            layout.uuid,
            self.source.label,
            self.source.uuid
        );

        Ok(RawQueryable::new(
            senders,
            receivers,
            self.clock.clone(),
            self.source.clone(),
            layout,
        ))
    }

    /// Creates a new typed Queryable, this queryable has type information
    pub async fn with<T: ArrowMessage, F: ArrowMessage>(
        &mut self,
        queryable: impl Into<String>,
    ) -> Result<Queryable<T, F>> {
        let (senders, receivers, layout) = self.compute(queryable).await?;

        tracing::debug!(
            "Creating new queryable '{}' (uuid: {}) for node '{}' (uuid: {})",
            layout.label,
            layout.uuid,
            self.source.label,
            self.source.uuid
        );

        Ok(Queryable::new(
            senders,
            receivers,
            self.clock.clone(),
            self.source.clone(),
            layout,
        ))
    }
}
