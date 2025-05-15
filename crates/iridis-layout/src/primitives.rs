//! This module defines all the primitive IDs used in the `dataflow` application.

use crate::prelude::*;

/// Represents a unique identifier for an Input in the graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputID {
    pub label: String,
    pub uuid: Uuid,
}

/// Represents a unique identifier for an Output in the graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OutputID {
    pub label: String,
    pub uuid: Uuid,
}

/// Represents a unique identifier for a Query in the graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QueryID {
    pub label: String,
    pub uuid: Uuid,
}

/// Represents a unique identifier for a Queryable in the graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QueryableID {
    pub label: String,
    pub uuid: Uuid,
}

/// Represents a unique identifier for a primitive in the graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PrimitiveID {
    Input(InputID),
    Output(OutputID),
    Query(QueryID),
    Queryable(QueryableID),
}

macro_rules! impl_into_primitive {
    ($($ty:ty => $variant:ident),*) => {
        $(
            impl From<$ty> for PrimitiveID {
                fn from(value: $ty) -> PrimitiveID {
                    PrimitiveID::$variant(value)
                }
            }

            impl From<&$ty> for PrimitiveID {
                fn from(value: &$ty) -> PrimitiveID {
                    PrimitiveID::$variant(value.clone())
                }
            }
        )*
    };
}

impl_into_primitive!(
    InputID => Input,
    OutputID => Output,
    QueryID => Query,
    QueryableID => Queryable
);

macro_rules! impl_into_uuid {
    ($($ty:ty),*) => {
        $(
            impl From<$ty> for Uuid {
                fn from(value: $ty) -> Uuid {
                    value.uuid
                }
            }

            impl From<&$ty> for Uuid {
                fn from(value: &$ty) -> Uuid {
                    value.uuid
                }
            }
        )*
    };
}

impl_into_uuid!(InputID, OutputID, QueryID, QueryableID);

impl PrimitiveID {
    pub fn label(&self) -> String {
        match self {
            PrimitiveID::Input(input) => input.label.clone(),
            PrimitiveID::Output(output) => output.label.clone(),
            PrimitiveID::Query(query) => query.label.clone(),
            PrimitiveID::Queryable(queryable) => queryable.label.clone(),
        }
    }

    pub fn uuid(&self) -> Uuid {
        match self {
            PrimitiveID::Input(input) => input.uuid,
            PrimitiveID::Output(output) => output.uuid,
            PrimitiveID::Query(query) => query.uuid,
            PrimitiveID::Queryable(queryable) => queryable.uuid,
        }
    }
}

impl TryFrom<PrimitiveID> for InputID {
    type Error = eyre::Report;

    fn try_from(value: PrimitiveID) -> eyre::Result<Self> {
        match value {
            PrimitiveID::Input(layout) => Ok(layout),
            _ => Err(eyre::eyre!("Invalid type")),
        }
    }
}

impl TryFrom<PrimitiveID> for OutputID {
    type Error = eyre::Report;

    fn try_from(value: PrimitiveID) -> eyre::Result<Self> {
        match value {
            PrimitiveID::Output(layout) => Ok(layout),
            _ => Err(eyre::eyre!("Invalid type")),
        }
    }
}

impl TryFrom<PrimitiveID> for QueryID {
    type Error = eyre::Report;

    fn try_from(value: PrimitiveID) -> eyre::Result<Self> {
        match value {
            PrimitiveID::Query(layout) => Ok(layout),
            _ => Err(eyre::eyre!("Invalid type")),
        }
    }
}

impl TryFrom<PrimitiveID> for QueryableID {
    type Error = eyre::Report;

    fn try_from(value: PrimitiveID) -> eyre::Result<Self> {
        match value {
            PrimitiveID::Queryable(layout) => Ok(layout),
            _ => Err(eyre::eyre!("Invalid type")),
        }
    }
}

impl From<PrimitiveID> for Uuid {
    fn from(val: PrimitiveID) -> Self {
        match val {
            PrimitiveID::Input(input) => input.uuid,
            PrimitiveID::Output(output) => output.uuid,
            PrimitiveID::Query(query) => query.uuid,
            PrimitiveID::Queryable(queryable) => queryable.uuid,
        }
    }
}

impl AsRef<Uuid> for PrimitiveID {
    fn as_ref(&self) -> &Uuid {
        match self {
            PrimitiveID::Input(input) => &input.uuid,
            PrimitiveID::Output(output) => &output.uuid,
            PrimitiveID::Query(query) => &query.uuid,
            PrimitiveID::Queryable(queryable) => &queryable.uuid,
        }
    }
}

impl From<&PrimitiveID> for Uuid {
    fn from(val: &PrimitiveID) -> Self {
        match val {
            PrimitiveID::Input(input) => input.uuid,
            PrimitiveID::Output(output) => output.uuid,
            PrimitiveID::Query(query) => query.uuid,
            PrimitiveID::Queryable(queryable) => queryable.uuid,
        }
    }
}
