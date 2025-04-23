use std::process::exit;
use crate::models::client::Client;
use crate::models::common::command::Command;
use crate::models::common::message::Message;
use crate::models::common::short_peer_id::ShortPeerId;
use crate::models::swarm::swarm_manager::SwarmManager;
use anyhow::Result;
use tokio::sync::mpsc;

const BUFFER_SIZE: usize = 30;

pub struct App {
    client: Client,
    swarm_manager: SwarmManager,
}

impl App {
    pub fn new() -> Result<Self> {
        let (command_sender, command_receiver) = mpsc::channel::<Command>(BUFFER_SIZE);

        let (msg_sender, msg_receiver) = mpsc::channel::<(Message, ShortPeerId)>(BUFFER_SIZE);

        let client = Client::new(msg_receiver, command_sender);

        let swarm_manager = SwarmManager::build(msg_sender, command_receiver)?;

        Ok(Self {
            client,
            swarm_manager,
        })
    }

    pub async fn run(mut self) {
        let swarm_task = tokio::spawn(async move {
            self.swarm_manager.run().await;
        });

        let client_task = tokio::spawn(async move {
            self.client.run().await;
        });

        let (swarm_res, client_res) = tokio::join!(swarm_task, client_task);

        if let Err(e) = swarm_res {
            log::error!("Error in swarm_manager: {e}. Shutting down...");
            exit(1);
        }

        if let Err(e) = client_res {
            log::error!("Error in client: {e}. Shutting down...");
            exit(2);
        }
    }
}
