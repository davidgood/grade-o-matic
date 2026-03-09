use libgrader::common::config::{Config, setup_database};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = Config::from_env()?;
    let _pool = setup_database(&config).await?;

    info!("grader service started");

    tokio::signal::ctrl_c().await?;
    info!("grader service stopping");

    Ok(())
}
