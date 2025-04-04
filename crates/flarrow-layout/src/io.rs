use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OutputID(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InputID(pub Uuid);
