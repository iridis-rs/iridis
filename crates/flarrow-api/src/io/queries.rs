use eyre::{Context, OptionExt};
use std::{collections::HashMap, sync::Arc};

use tokio::sync::{
    Mutex,
    mpsc::{Receiver, Sender},
};

use arrow_array::Array;
use arrow_data::ArrayData;

use crate::prelude::*;

pub struct RawQuery {
    clock: Arc<uhlc::HLC>,
    uuid: QueryUUID,

    pub tx: Sender<DataflowMessage>,
    pub rx: Receiver<DataflowMessage>,
}

impl RawQuery {
    pub fn new(
        clock: Arc<uhlc::HLC>,
        uuid: QueryUUID,
        tx: Sender<DataflowMessage>,
        rx: Receiver<DataflowMessage>,
    ) -> Self {
        Self {
            tx,
            rx,
            clock,
            uuid,
        }
    }

    pub fn query(&mut self, data: ArrayData) -> eyre::Result<(Header, ArrayData)> {
        let data = DataflowMessage {
            header: Header {
                timestamp: self.clock.new_timestamp(),
                node: None,
                source: Some(Source::Query(self.uuid)),
            },
            data,
        };

        self.tx
            .blocking_send(data)
            .wrap_err("Failed to send to this output")?;

        let DataflowMessage { header, data } = self
            .rx
            .blocking_recv()
            .ok_or_eyre("Failed to receive from this input")?;

        Ok((header, data))
    }

    pub async fn query_async(&mut self, data: ArrayData) -> eyre::Result<(Header, ArrayData)> {
        let data = DataflowMessage {
            header: Header {
                timestamp: self.clock.new_timestamp(),
                node: None,
                source: Some(Source::Query(self.uuid)),
            },
            data,
        };

        self.tx
            .send(data)
            .await
            .wrap_err("Failed to send to this output")?;

        let DataflowMessage { header, data } = self
            .rx
            .recv()
            .await
            .ok_or_eyre("Failed to receive from this input")?;

        Ok((header, data))
    }
}

pub struct Query<T: ArrowMessage, F: ArrowMessage> {
    pub raw: RawQuery,

    _phantom: std::marker::PhantomData<(T, F)>,
}

impl<T: ArrowMessage, F: ArrowMessage> Query<T, F> {
    pub fn new(
        clock: Arc<uhlc::HLC>,
        uuid: QueryUUID,
        tx: Sender<DataflowMessage>,
        rx: Receiver<DataflowMessage>,
    ) -> Self {
        Self {
            raw: RawQuery::new(clock, uuid, tx, rx),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn query(&mut self, data: T) -> eyre::Result<(Header, F)> {
        let (header, data) = self.raw.query(
            data.try_into_arrow()
                .wrap_err("Failed to convert arrow 'data' to message T")?
                .into_data(),
        )?;

        Ok((
            header,
            F::try_from_arrow(data).wrap_err("Failed to convert arrow 'data' to message T")?,
        ))
    }

    pub async fn query_async(&mut self, data: T) -> eyre::Result<(Header, F)> {
        let (header, data) = self
            .raw
            .query_async(
                data.try_into_arrow()
                    .wrap_err("Failed to convert arrow 'data' to message T")?
                    .into_data(),
            )
            .await?;

        Ok((
            header,
            F::try_from_arrow(data).wrap_err("Failed to convert arrow 'data' to message T")?,
        ))
    }
}

pub struct Queries {
    node: NodeUUID,

    clock: Arc<uhlc::HLC>,

    #[allow(clippy::type_complexity)]
    senders: Arc<Mutex<HashMap<QueryUUID, Sender<DataflowMessage>>>>,
    #[allow(clippy::type_complexity)]
    receivers: Arc<Mutex<HashMap<QueryUUID, Receiver<DataflowMessage>>>>,
}

impl Queries {
    #[allow(clippy::type_complexity)]
    pub fn new(
        node: NodeUUID,
        clock: Arc<uhlc::HLC>,
        senders: Arc<Mutex<HashMap<QueryUUID, Sender<DataflowMessage>>>>,
        receivers: Arc<Mutex<HashMap<QueryUUID, Receiver<DataflowMessage>>>>,
    ) -> Self {
        Self {
            node,
            clock,
            senders,
            receivers,
        }
    }

    pub async fn raw(&mut self, query: impl Into<String>) -> eyre::Result<RawQuery> {
        let id = self.node.query(query);

        let sender = self
            .senders
            .lock()
            .await
            .remove(&id)
            .ok_or_eyre(format!("Input {} not found", id.0))?;

        let receiver = self
            .receivers
            .lock()
            .await
            .remove(&id)
            .ok_or_eyre(format!("Input {} not found", id.0))?;

        Ok(RawQuery::new(self.clock.clone(), id, sender, receiver))
    }

    pub async fn with<T: ArrowMessage, F: ArrowMessage>(
        &mut self,
        query: impl Into<String>,
    ) -> eyre::Result<Query<T, F>> {
        let id = self.node.query(query);

        let sender = self
            .senders
            .lock()
            .await
            .remove(&id)
            .ok_or_eyre(format!("Input {} not found", id.0))?;

        let receiver = self
            .receivers
            .lock()
            .await
            .remove(&id)
            .ok_or_eyre(format!("Input {} not found", id.0))?;

        Ok(Query::new(self.clock.clone(), id, sender, receiver))
    }
}
