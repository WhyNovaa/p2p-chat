use std::path::PathBuf;
use tokio::io;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{mpsc, oneshot};
use crate::models::client::command::Command;
use crate::models::swarm::message::Message;

pub struct Client {
    command_sender: mpsc::Sender<Command>,
    msg_receiver: mpsc::Receiver<Message>,
}

impl Client {
    pub fn new(msg_receiver: mpsc::Receiver<Message>, command_sender: mpsc::Sender<Command>) -> Self {

        Self {
            command_sender,
            msg_receiver,
        }
    }

    pub async fn run(mut self) {
        let stdin = io::stdin();
        let reader = BufReader::new(stdin);
        let mut lines = reader.lines();

        tokio::spawn(async move {
            loop {
                if let Ok(response) = self.msg_receiver.try_recv() {
                    println!("{:?}", response);
                }
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        });

        while let Ok(Some(line)) = lines.next_line().await {
            let msg = Message::build(Some(line), None::<PathBuf>).await.unwrap();
            let (response_sender, mut response_receiver) = oneshot::channel::<String>();
            let command = Command::SendMessage { response_sender, msg };
            self.command_sender.send(command).await.unwrap();
            for i in 0..10 {
                println!("{i}");
                if let Ok(response) = response_receiver.try_recv() {
                    println!("{}", response);
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        }
    }
}