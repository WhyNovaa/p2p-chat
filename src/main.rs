use anyhow::Result;
use crate::models::app::App;

mod models;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    App::new()?.run().await;

    Ok(())
}



