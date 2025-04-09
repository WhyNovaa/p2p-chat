use std::path::PathBuf;
use tokio::io;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{mpsc, oneshot};
use crate::models::common::command::Command;
use crate::models::common::message::Message;
use crate::models::common::short_peer_id::ShortPeerId;

pub struct Client {
    command_sender: mpsc::Sender<Command>,
    /// message and first 8 sender PeerId symbols
    msg_receiver: mpsc::Receiver<(Message, ShortPeerId)>,
}

impl Client {
    pub fn new(msg_receiver: mpsc::Receiver<(Message, ShortPeerId)>, command_sender: mpsc::Sender<Command>) -> Self {
        Self {
            command_sender,
            msg_receiver,
        }
    }

    pub async fn start_receiving(mut msg_receiver: mpsc::Receiver<(Message, ShortPeerId)>) {
        tokio::spawn(async move {
            loop {
                if let Ok(response) = msg_receiver.try_recv() {
                    println!("{}: {}", response.1, response.0);
                }
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        });
    }

    pub async fn start_writing(command_sender: mpsc::Sender<Command>) {
        let stdin = io::stdin();
        let reader = BufReader::new(stdin);
        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            let msg = Message::build(Some(line), None::<PathBuf>).await.unwrap();
            let (response_sender, mut response_receiver) = oneshot::channel::<String>();
            let command = Command::SendMessage { response_sender, msg };
            command_sender.send(command).await.unwrap();
            for _ in 0..10 {
                if let Ok(response) = response_receiver.try_recv() {
                    println!("{}", response);
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        }

    }

    pub async fn run(self) {
        Self::start_receiving(self.msg_receiver).await;

        Self::start_writing(self.command_sender).await;
    }
}