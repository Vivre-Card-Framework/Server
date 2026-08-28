use anyhow::{Context, Result};
use chrono::Utc;
use etcd_client::{Client, ConnectOptions, PutOptions};
use std::time::Duration;
use tokio::sync::OnceCell;

use crate::credential;

static ETCD_CLIENT: OnceCell<Client> = OnceCell::const_new();

pub async fn init() -> Result<()> {
    ETCD_CLIENT
        .get_or_try_init(|| async {
            let loaded = credential::get()?;
            let endpoint = loaded.endpoint.clone();
            let username = loaded.server_id.clone();
            let password = loaded.password.clone();
            let state_key = format!("/v1/{}/_state", loaded.server_id);
            let last_seen_key = format!("/v1/{}/_last_seen", loaded.server_id);

            let options = ConnectOptions::new()
                .with_user(username, password)
                .with_connect_timeout(Duration::from_secs(5))
                .with_timeout(Duration::from_secs(10))
                .with_keep_alive(Duration::from_secs(10), Duration::from_secs(3))
                .with_keep_alive_while_idle(true);

            let mut client = Client::connect([endpoint], Some(options))
                .await
                .context("failed to connect to etcd")?;

            client
                .get(state_key.clone(), None)
                .await
                .context("failed to verify Server etcd credentials")?;

            let lease = client
                .lease_grant(15, None)
                .await
                .context("failed to create Server state lease")?;
            let lease_id = lease.id();
            client
                .put(
                    state_key,
                    "ready",
                    Some(PutOptions::new().with_lease(lease_id)),
                )
                .await
                .context("failed to register Server state")?;

            client
                .put(last_seen_key.clone(), Utc::now().to_rfc3339(), None)
                .await
                .context("failed to register Server last-seen time")?;

            let (mut keeper, mut stream) = client
                .lease_keep_alive(lease_id)
                .await
                .context("failed to start Server state lease keepalive")?;
            let mut heartbeat_client = client.clone();
            tokio::spawn(async move {
                loop {
                    if let Err(error) = keeper.keep_alive().await {
                        tracing::error!(%error, "Server state lease keepalive failed");
                        break;
                    }
                    match stream.message().await {
                        Ok(Some(_)) => {}
                        Ok(None) => {
                            tracing::error!("Server state lease keepalive stream closed");
                            break;
                        }
                        Err(error) => {
                            tracing::error!(%error, "Server state lease response failed");
                            break;
                        }
                    }

                    if let Err(error) = heartbeat_client
                        .put(last_seen_key.clone(), Utc::now().to_rfc3339(), None)
                        .await
                    {
                        tracing::warn!(%error, "failed to update Server last-seen time");
                    }
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            });

            Ok::<Client, anyhow::Error>(client)
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
