use eyre::eyre;

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Builtin {
    Timer,
}

impl Builtin {
    pub fn from_string(str: impl AsRef<str>) -> Result<Self> {
        match str.as_ref() {
            "timer" => Ok(Builtin::Timer),
            _ => Err(eyre!("Invalid builtin name")),
        }
    }
}

pub async fn new_builtin(
    builtin: Builtin,
    inputs: Inputs,
    outputs: Outputs,
    configuration: serde_yml::Value,
) -> Result<Box<dyn Node>> {
    match builtin {
        Builtin::Timer => Timer::new(inputs, outputs, configuration)
            .await
            .wrap_err("Failed to await statically linked node")?,
    }
}
