use crate::models::app::App;
use anyhow::Result;

mod models;
mod traits;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    App::new()?.run().await;

    Ok(())
}
