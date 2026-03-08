pub mod poll;

use libgrader::common::config::{Config, setup_database};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = Config::from_env()?;
    let pool = setup_database(&config).await?;

    info!("grader service started");
    poll::run(pool, config).await?;
    info!("grader service stopping");

    Ok(())
}
