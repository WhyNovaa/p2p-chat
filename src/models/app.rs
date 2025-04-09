use tokio::sync::mpsc;
use crate::models::client::client::Client;
use crate::models::client::command::Command;
use crate::models::swarm::message::Message;
use crate::models::swarm::swarm_manager::SwarmManager;
use anyhow::Result;
use crate::models::swarm::short_peer_id::ShortPeerId;

pub struct App {
    client: Client,
    swarm_manager: SwarmManager,
}

impl App {
    pub fn new() -> Result<Self> {
        let (command_sender, command_receiver) = mpsc::channel::<Command>(30);

        let (msg_sender, msg_receiver) = mpsc::channel::<(Message, ShortPeerId)>(30);

        let client = Client::new(msg_receiver, command_sender);

        let swarm_manager = SwarmManager::build(msg_sender, command_receiver)?.with_topic("test");

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

        let _ = tokio::join!(swarm_task, client_task);
    }
}