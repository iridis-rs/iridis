use crate::prelude::*;

/// Represents an input
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputLayout {
    pub label: String,
    pub uuid: Uuid,
}

/// Represents an output
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OutputLayout {
    pub label: String,
    pub uuid: Uuid,
}

/// Represents a query
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QueryLayout {
    pub label: String,
    pub uuid: Uuid,
}

/// Represents a queryable
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QueryableLayout {
    pub label: String,
    pub uuid: Uuid,
}

/// Represents a kind of IO (Input, Output, Query, Queryable)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IOLayout {
    Input(InputLayout),
    Output(OutputLayout),
    Query(QueryLayout),
    Queryable(QueryableLayout),
}

macro_rules! impl_into_nodeio {
    ($($ty:ty => $variant:ident),*) => {
        $(
            impl From<$ty> for IOLayout {
                fn from(value: $ty) -> IOLayout {
                    IOLayout::$variant(value)
                }
            }

            impl From<&$ty> for IOLayout {
                fn from(value: &$ty) -> IOLayout {
                    IOLayout::$variant(value.clone())
                }
            }
        )*
    };
}

impl_into_nodeio!(
    InputLayout => Input,
    OutputLayout => Output,
    QueryLayout => Query,
    QueryableLayout => Queryable
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

impl_into_uuid!(InputLayout, OutputLayout, QueryLayout, QueryableLayout);

impl IOLayout {
    /// Returns the label of the IOLayout.
    pub fn label(&self) -> String {
        match self {
            IOLayout::Input(input) => input.label.clone(),
            IOLayout::Output(output) => output.label.clone(),
            IOLayout::Query(query) => query.label.clone(),
            IOLayout::Queryable(queryable) => queryable.label.clone(),
        }
    }

    /// Returns the UUID of the IOLayout.
    pub fn uuid(&self) -> Uuid {
        match self {
            IOLayout::Input(input) => input.uuid,
            IOLayout::Output(output) => output.uuid,
            IOLayout::Query(query) => query.uuid,
            IOLayout::Queryable(queryable) => queryable.uuid,
        }
    }
}

impl TryFrom<IOLayout> for InputLayout {
    type Error = eyre::Report;

    fn try_from(value: IOLayout) -> eyre::Result<Self> {
        match value {
            IOLayout::Input(layout) => Ok(layout),
            _ => Err(eyre::eyre!("Invalid type")),
        }
    }
}

impl TryFrom<IOLayout> for OutputLayout {
    type Error = eyre::Report;

    fn try_from(value: IOLayout) -> eyre::Result<Self> {
        match value {
            IOLayout::Output(layout) => Ok(layout),
            _ => Err(eyre::eyre!("Invalid type")),
        }
    }
}

impl TryFrom<IOLayout> for QueryLayout {
    type Error = eyre::Report;

    fn try_from(value: IOLayout) -> eyre::Result<Self> {
        match value {
            IOLayout::Query(layout) => Ok(layout),
            _ => Err(eyre::eyre!("Invalid type")),
        }
    }
}

impl TryFrom<IOLayout> for QueryableLayout {
    type Error = eyre::Report;

    fn try_from(value: IOLayout) -> eyre::Result<Self> {
        match value {
            IOLayout::Queryable(layout) => Ok(layout),
            _ => Err(eyre::eyre!("Invalid type")),
        }
    }
}

impl From<IOLayout> for Uuid {
    fn from(val: IOLayout) -> Self {
        match val {
            IOLayout::Input(input) => input.uuid,
            IOLayout::Output(output) => output.uuid,
            IOLayout::Query(query) => query.uuid,
            IOLayout::Queryable(queryable) => queryable.uuid,
        }
    }
}

impl From<&IOLayout> for Uuid {
    fn from(val: &IOLayout) -> Self {
        match val {
            IOLayout::Input(input) => input.uuid,
            IOLayout::Output(output) => output.uuid,
            IOLayout::Query(query) => query.uuid,
            IOLayout::Queryable(queryable) => queryable.uuid,
        }
    }
}
