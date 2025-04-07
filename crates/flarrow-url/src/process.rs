use std::path::PathBuf;

use eyre::eyre;
use reqwest::Client;
use tokio::io::AsyncWriteExt;
use url::Url;
use uuid::Uuid;

use crate::prelude::*;

pub async fn process_url(url: Url) -> Result<NodeKind> {
    match url.scheme() {
        "builtin" => {
            let builtin_name = url.path().trim_start_matches('/').to_string();
            Ok(NodeKind::Builtin(
                Builtin::from_string(builtin_name).wrap_err("Failed to parse builtin name")?,
            ))
        }
        "file" => {
            let path = PathBuf::from(url.path());
            if let Some(ext) = path.extension() {
                if ext == "py" {
                    Ok(NodeKind::PythonScript(path))
                } else if ext == std::env::consts::DLL_EXTENSION {
                    Ok(NodeKind::DynamicallyLinkedLibrary(path))
                } else {
                    Err(eyre!("Unsupported file extension"))
                }
            } else {
                Err(eyre!("No file extension found"))
            }
        }
        "http" | "https" => {
            let client = Client::new();
            let response = client.get(url.as_str()).send().await?.bytes().await?;

            let temp_dir = std::env::temp_dir();
            let file_name = url
                .path_segments()
                .and_then(|mut segments| segments.next_back())
                .ok_or(eyre!("No file name in URL: {}", url))?;
            let unique_file_name = format!("{}-{}", Uuid::new_v4(), file_name);
            let temp_path = temp_dir.join(unique_file_name);

            let mut file = tokio::fs::File::create(&temp_path).await?;
            file.write_all(&response).await?;

            if let Some(ext) = temp_path.extension() {
                if ext == "py" {
                    Ok(NodeKind::PythonScript(temp_path))
                } else if ext == std::env::consts::DLL_EXTENSION {
                    Ok(NodeKind::DynamicallyLinkedLibrary(temp_path))
                } else {
                    // TODO: support extracting .tar.gz
                    Err(eyre!(
                        "Unsupported file extension: {}",
                        ext.to_string_lossy()
                    ))
                }
            } else {
                Err(eyre!("No file extension found"))
            }
        }
        _ => Err(eyre!("Unsupported URL scheme: {}", url.scheme())),
    }
}
