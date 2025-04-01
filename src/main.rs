use crate::models::swarm_manager::SwarmManager;
use anyhow::Result;
mod models;

#[tokio::main]
async fn main() -> Result<()> {
    SwarmManager::new()?.run().await?;

    Ok(())
}



