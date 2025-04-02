use tokio::sync::mpsc;
use crate::models::client::command::Command;
use crate::models::swarm::message::Message;

pub struct Client {
    sender: mpsc::Sender<Command>,
    msg_receiver: mpsc::Receiver<Message>,
}

impl Client {
    pub fn new(msg_receiver: mpsc::Receiver<Message>) -> (Self, mpsc::Receiver<Command>) {
        let (sender, receiver) = mpsc::channel(15);

        let client = Self {
            sender,
            msg_receiver,
        };

        (client, receiver)
    }

    pub async fn start(&self) {
        //todo
    }
}