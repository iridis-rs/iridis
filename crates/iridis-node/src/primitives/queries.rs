//! This module contains implementations for this primitive.

use std::{collections::HashMap, sync::Arc};

use crate::prelude::{thirdparty::tokio::sync::Mutex, *};

type SharedMap<K, V> = Arc<Mutex<HashMap<K, V>>>;
type Senders = SharedMap<Uuid, MessageSender>;
type Receivers = SharedMap<Uuid, MessageReceiver>;

/// Queries let you manage query connections during a node *implementation*
pub struct Queries {
    senders: Senders,
    receivers: Receivers,

    clock: Arc<uhlc::HLC>,

    source: NodeID,
}

impl Queries {
    /// Creates a new instance of `Queries`.
    pub fn new(
        senders: Senders,
        receivers: Receivers,
        clock: Arc<uhlc::HLC>,
        source: NodeID,
    ) -> Self {
        Self {
            senders,
            receivers,
            clock,
            source,
        }
    }

    async fn compute(
        &mut self,
        query: impl Into<String>,
    ) -> Result<(MessageSender, MessageReceiver, QueryID)> {
        let label: String = query.into();
        let layout = self.source.query(&label);

        let sender = self
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

        Ok((sender, receiver, layout))
    }

    /// Creates a new raw Query, this raw query has no type information so you have
    /// to manually transform it
    pub async fn raw(&mut self, query: impl Into<String>) -> Result<RawQuery> {
        let (tx, rx, layout) = self.compute(query).await?;

        tracing::debug!(
            "Creating new raw query '{}' (uuid: {}) for node '{}' (uuid: {})",
            layout.label,
            layout.uuid,
            self.source.label,
            self.source.uuid
        );

        Ok(RawQuery::new(
            tx,
            rx,
            self.clock.clone(),
            self.source.clone(),
            layout,
        ))
    }

    /// Creates a new query, this query has type information
    pub async fn with<T: ArrowMessage, F: ArrowMessage>(
        &mut self,
        query: impl Into<String>,
    ) -> Result<Query<T, F>> {
        let (tx, rx, layout) = self.compute(query).await?;

        tracing::debug!(
            "Creating new query '{}' (uuid: {}) for node '{}' (uuid: {})",
            layout.label,
            layout.uuid,
            self.source.label,
            self.source.uuid
        );

        Ok(Query::new(
            tx,
            rx,
            self.clock.clone(),
            self.source.clone(),
            layout,
        ))
    }
}
