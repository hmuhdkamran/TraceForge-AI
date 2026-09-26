use contractguard_lib::{app, config::Config};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "contractguard=debug,info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env()?;
    let bind_addr = config.bind_addr.clone();

    tracing::info!("Starting ContractGuard backend on {}", bind_addr);

    let router = app(config).await?;

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!("Listening on {}", bind_addr);

    axum::serve(listener, router).await?;

    Ok(())
}
