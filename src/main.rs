use anyhow::Result;
use tokio::sync::mpsc;
use crate::models::client::command::Command;
use crate::models::swarm::swarm_manager::SwarmManager;
use tokio::sync::oneshot;
use crate::models::client::client::Client;
use crate::models::swarm::message::Message;

mod models;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    tokio::spawn(async move {
        let (msg_s, msg_r) = mpsc::channel::<Message>(10);
        let (client, rec) = Client::new(msg_r);
        if let Ok(mut manager) = SwarmManager::new(msg_s, rec) {
            if let Err(e) = manager.run().await {
                eprintln!("SwarmManager error: {}", e);
            }
        }

    }).await?;

    Ok(())
}



