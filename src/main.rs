pub mod router;

use std::env;
use tokio::net::TcpListener;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

fn parse_log_level() -> Level {
    let level = env::var("log_level")
        .ok()
        .and_then(|v| v.parse::<u8>().ok())
        .unwrap_or(2); // default INFO

    match level {
        0 => Level::TRACE,
        1 => Level::DEBUG,
        2 => Level::INFO,
        3 => Level::WARN,
        4 => Level::ERROR,
        _ => Level::INFO,
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // load env
    dotenvy::dotenv().ok();

    // set tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(parse_log_level())
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let listener = TcpListener::bind(env::var("listener")?).await?;
    tracing::info!(
        "vivre card framework server started at {}",
        env::var("listener")?
    );
    axum::serve(listener, router::get()).await?;

    Ok(())
}
