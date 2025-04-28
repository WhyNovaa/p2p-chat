use crate::models::common::message::Message;
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum Command {
    SendMessage {
        response_sender: oneshot::Sender<String>,
        msg: Message,
    },
    Subscribe {
        response_sender: oneshot::Sender<String>,
        topic_name: String,
    },
    Unsubscribe {
        response_sender: oneshot::Sender<String>,
        topic_name: String,
    },
}

impl Command {
    pub fn new_send_message(msg: Message) -> (Self, oneshot::Receiver<String>) {
        let (response_sender, response_receiver) = oneshot::channel::<String>();

        let command = Command::SendMessage {
            response_sender,
            msg,
        };

        (command, response_receiver)
    }

    pub fn new_subscribe(topic_name: String) -> (Self, oneshot::Receiver<String>) {
        let (response_sender, response_receiver) = oneshot::channel::<String>();

        let command = Command::Subscribe {
            response_sender,
            topic_name,
        };

        (command, response_receiver)
    }

    pub fn new_unsubscribe(topic_name: String) -> (Self, oneshot::Receiver<String>) {
        let (response_sender, response_receiver) = oneshot::channel::<String>();

        let command = Command::Unsubscribe {
            response_sender,
            topic_name,
        };

        (command, response_receiver)
    }
}
