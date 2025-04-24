use crate::prelude::*;

/// Enumeration of built-in nodes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Builtin {
    Timer,
    Printer,
    Transport,
}

impl Builtin {
    /// Creates a new instance of a built-in node from a string.
    pub fn from_string(str: impl AsRef<str>) -> Result<Self> {
        match str.as_ref() {
            "timer" => Ok(Builtin::Timer),
            "printer" => Ok(Builtin::Printer),
            "transport" => Ok(Builtin::Transport),
            _ => Err(eyre::eyre!("Invalid builtin name")),
        }
    }
}

/// Creates a new instance of a StaticallyLinked RuntimeNode from a Builtin enum.
pub async fn new_builtin(
    builtin: Builtin,
    inputs: Inputs,
    outputs: Outputs,
    queries: Queries,
    queryables: Queryables,
    configuration: serde_yml::Value,
) -> Result<Box<dyn Node>> {
    match builtin {
        Builtin::Timer => Timer::new(inputs, outputs, queries, queryables, configuration)
            .await
            .wrap_err("Failed to await statically linked node")?,
        Builtin::Printer => Printer::new(inputs, outputs, queries, queryables, configuration)
            .await
            .wrap_err("Failed to await statically linked node")?,
        Builtin::Transport => Transport::new(inputs, outputs, queries, queryables, configuration)
            .await
            .wrap_err("Failed to await statically linked node")?,
    }
}
