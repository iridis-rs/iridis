use std::path::PathBuf;

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeKind {
    Builtin(Builtin),
    DynamicallyLinkedLibrary(PathBuf),
    PythonScript(PathBuf),
}
