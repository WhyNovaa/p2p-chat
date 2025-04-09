use tokio::sync::oneshot;
use crate::models::common::message::Message;

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
    }
}