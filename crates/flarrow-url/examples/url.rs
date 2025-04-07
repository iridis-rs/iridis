use flarrow_url::prelude::*;
use url::Url;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // let url = "builtin:///timer";
    let url = "https://raw.githubusercontent.com/dora-rs/dora-lerobot/refs/heads/main/node-hub/lebai-client/lebai_client/main.py";
    let url = Url::parse(url)?;

    let node = process_url(url).await?;
    println!("{:?}", node);
    Ok(())
}
