use anyhow::{Context, Result};
use etcd_client::{Client, ConnectOptions};
use std::time::Duration;
use tokio::sync::OnceCell;

static ETCD_CLIENT: OnceCell<Client> = OnceCell::const_new();

pub async fn init() -> Result<()> {
    ETCD_CLIENT
        .get_or_try_init(|| async {
            let endpoint =
                std::env::var("etcd_url").context("missing etcd_url environment variable")?;

            let options = ConnectOptions::new()
                .with_connect_timeout(Duration::from_secs(5))
                .with_timeout(Duration::from_secs(10))
                .with_keep_alive(Duration::from_secs(10), Duration::from_secs(3))
                .with_keep_alive_while_idle(true);

            Client::connect([endpoint], Some(options))
                .await
                .context("failed to connect to etcd")
        })
        .await?;

    Ok(())
}

pub fn client() -> Result<Client> {
    ETCD_CLIENT
        .get()
        .cloned()
        .context("etcd client is not initialized")
}
