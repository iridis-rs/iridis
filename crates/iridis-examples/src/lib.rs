#[cfg(feature = "default")]
use iridis::prelude::thirdparty::*;

#[cfg(feature = "default")]
pub fn dylib(name: &str, build: Option<&str>) -> Result<Url> {
    let path = std::env::var("CARGO_MANIFEST_DIR")?;
    let path = format!(
        "file://{}/../../target/{}/examples",
        path,
        build.unwrap_or("debug")
    );

    let prefix = std::env::consts::DLL_PREFIX;
    let dylib = std::env::consts::DLL_SUFFIX;

    Url::parse(&format!("{}/{}{}{}", path, prefix, name, dylib)).map_err(eyre::Report::msg)
}
