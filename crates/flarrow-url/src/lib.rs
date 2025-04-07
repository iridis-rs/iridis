pub(crate) mod kind;
pub(crate) mod process;

/// Get node data from a URL: (TemporaryPath of this file, Kind (Builtin, DLL, Python))
/// - builtin://builtin_name will return Builtin(builtin_name)
/// - file://path/to/a/file will return either DynamicallyLinkedLibrary(path) or PythonScript(path) according to the file extension
/// - http://path/to/a/file will will download the file to a temporary location and return either DynamicallyLinkedLibrary(path) or PythonScript(path) according to the file extension
/// - https://path/to/a/file will will download the file to a temporary location and return DynamicallyLinkedLibrary(path)
pub mod prelude {
    pub use crate::kind::*;
    pub use crate::process::*;

    pub use flarrow_builtins::prelude::*;
}
