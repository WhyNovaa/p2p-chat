use crate::models::common::command::Command;
use crate::models::common::errors::SendingError;
use crate::models::common::message::Message;
use crate::models::common::short_peer_id::ShortPeerId;
use crate::models::swarm::behaviour::{ChatBehaviour, ChatBehaviourEvent};
use crate::traits::decode::Decode;
use crate::traits::encode::Encode;
use libp2p::futures::StreamExt;
use libp2p::gossipsub::IdentTopic;
use libp2p::mdns::Event;
use libp2p::swarm::SwarmEvent;
use libp2p::{Swarm, gossipsub, noise, tcp, yamux};
use tokio::sync::mpsc;

pub struct SwarmManager {
    swarm: Swarm<ChatBehaviour>,
    command_receiver: mpsc::Receiver<Command>,
    msg_sender: mpsc::Sender<(Message, ShortPeerId)>,
    current_topic: Option<IdentTopic>,
}

impl SwarmManager {
    pub fn build(
        msg_sender: mpsc::Sender<(Message, ShortPeerId)>,
        command_receiver: mpsc::Receiver<Command>,
    ) -> anyhow::Result<Self> {
        log::info!("Creating SwarmManager");
        let mut swarm = build_swarm()?;

        swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

        Ok(Self {
            swarm,
            command_receiver,
            msg_sender,
            current_topic: None,
        })
    }

    pub async fn run(&mut self) {
        log::info!("Running...");

        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => self.handle_event(event).await,
                command = self.command_receiver.recv() => self.handle_command(command).await,
            }
        }
    }

    pub fn with_topic(mut self, topic_name: impl Into<String>) -> Self {
        match self.subscribe(topic_name) {
            Ok(res) => match res {
                true => log::info!("Swarm subscribed to the topic successfully"),
                false => log::warn!("Swarm is already subscribed to to this topic"),
            },
            Err(e) => log::error!("Something went wrong: {e}"),
        };
        self
    }

    pub async fn handle_event(&mut self, event: SwarmEvent<ChatBehaviourEvent>) {
        match event {
            SwarmEvent::Behaviour(ChatBehaviourEvent::Mdns(mdns_event)) => match mdns_event {
                Event::Discovered(peers) => {
                    for (peer_id, _multiaddr) in peers {
                        log::info!("mDNS discovered a new peer: {peer_id}");
                        self.swarm
                            .behaviour_mut()
                            .gossipsub
                            .add_explicit_peer(&peer_id);
                    }
                }
                Event::Expired(peers) => {
                    for (peer_id, _multiaddr) in peers {
                        log::info!("mDNS discover peer has expired: {peer_id}");
                        self.swarm
                            .behaviour_mut()
                            .gossipsub
                            .remove_explicit_peer(&peer_id);
                    }
                }
            },
            SwarmEvent::Behaviour(ChatBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                propagation_source: peer_id,
                message_id: id,
                message,
            })) => {
                let msg = match Message::decode(&mut message.data.as_slice()) {
                    Ok(msg) => msg,
                    Err(e) => {
                        log::error!("Couldn't decode message: {e}");
                        return;
                    }
                };

                if msg.is_empty() {
                    return;
                }

                let sender_peer_id = ShortPeerId::from(&peer_id);

                log::info!("Got message: '{msg}' with id: {id} from peer: {peer_id}");
                match self.msg_sender.send((msg, sender_peer_id)).await {
                    Ok(_) => {}
                    Err(e) => log::error!("Couldn't send message to client: {e}"),
                }
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                log::info!("Local node is listening on {address}");
            }
            _ => {}
        }
    }

    async fn handle_command(&mut self, command: Option<Command>) {
        match command {
            Some(Command::SendMessage {
                response_sender,
                msg,
            }) => {
                log::info!("Sending message...");

                let ans = match self.send_message(msg) {
                    Ok(_) => "Ok".to_string(),
                    Err(SendingError::NoSubscribedTopic) => {
                        log::info!("Swarm not subscribed to any topic");
                        "You are not subscribed to any topic".to_string()
                    }
                    Err(SendingError::CantEncodeMessage) => {
                        log::info!("Can't encode message");
                        "Something wrong with the message".to_string()
                    }
                    Err(SendingError::Other(e)) => {
                        log::info!("Sending error: {e}");
                        "Something went wrong while sending message".to_string()
                    }
                };

                if let Err(e) = response_sender.send(ans) {
                    log::error!("Response wasn't sent to client: {e}");
                }
            }
            Some(Command::Subscribe {
                response_sender,
                topic_name,
            }) => {
                log::info!("Subscribing topic...");

                let ans = match self.subscribe(topic_name) {
                    Ok(res) => match res {
                        true => "You have subscribed to the topic successfully",
                        false => "You are already subscribed to to this topic",
                    },
                    Err(_) => "Something went wrong",
                };

                if let Err(e) = response_sender.send(ans.to_string()) {
                    log::error!("Response wasn't sent to client: {e}");
                };
            }
            Some(Command::Unsubscribe {
                response_sender,
                topic_name,
            }) => {
                log::info!("Unsubscribing topic...");

                let ans = match self.unsubscribe(topic_name) {
                    true => "You have unsubscribed from the topic successfully",
                    false => "You haven't been subscribed to this topic",
                };

                match response_sender.send(ans.to_string()) {
                    Ok(_) => {}
                    Err(e) => log::error!("Response wasn't sent to client: {e}"),
                };
            }
            None => {}
        }
    }

    pub fn subscribe(&mut self, topic_name: impl Into<String>) -> anyhow::Result<bool> {
        log::info!("Subscribing topic...");
        let topic = IdentTopic::new(topic_name);

        let res = self.swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

        self.current_topic = Some(topic);

        Ok(res)
    }

    pub fn unsubscribe(&mut self, topic_name: String) -> bool {
        log::info!("Unsubscribing topic...");
        let topic = IdentTopic::new(topic_name);

        let res = self.swarm.behaviour_mut().gossipsub.unsubscribe(&topic);

        self.current_topic = None;

        res
    }

    pub fn send_message(&mut self, message: Message) -> Result<(), SendingError> {
        if self.current_topic.is_none() {
            return Err(SendingError::NoSubscribedTopic);
        }

        let topic = self.current_topic.clone().unwrap();

        let encoded_message = message
            .encode_to_vec()
            .map_err(|_| SendingError::CantEncodeMessage)?;

        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(topic, encoded_message)?;

        Ok(())
    }
}

fn build_swarm() -> anyhow::Result<Swarm<ChatBehaviour>> {
    let swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_quic()
        .with_behaviour(|key| {
            let behaviour = ChatBehaviour::build(key)?;
            Ok(behaviour)
        })?
        .build();

    Ok(swarm)
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::gossipsub::PublishError;

    #[cfg(unix)]
    #[tokio::test]
    async fn communication_test() -> anyhow::Result<()> {
        let (command_sender1, command_receiver1) = mpsc::channel::<Command>(20);
        let (msg_sender1, _) = mpsc::channel::<(Message, ShortPeerId)>(20);

        let mut sw1 = SwarmManager::build(msg_sender1, command_receiver1)?.with_topic("1test");

        let (_, command_receiver2) = mpsc::channel::<Command>(20);
        let (msg_sender2, mut msg_receiver2) = mpsc::channel::<(Message, ShortPeerId)>(20);

        let mut sw2 = SwarmManager::build(msg_sender2, command_receiver2)?.with_topic("1test");

        for _ in 0..24 {
            let ev = sw1.swarm.select_next_some().await;
            sw1.handle_event(ev).await;
            let ev = sw2.swarm.select_next_some().await;
            sw2.handle_event(ev).await;
        }

        let msg = Message::build(Some("Test".to_string()), None).await;
        let (command, _) = Command::new_send_message(msg.clone());

        command_sender1.send(command).await?;

        let command = sw1.command_receiver.recv().await;
        sw1.handle_command(command).await;

        let ev = sw2.swarm.select_next_some().await;
        sw2.handle_event(ev).await;

        let (received_msg, _) = msg_receiver2.try_recv()?;

        assert_eq!(received_msg, msg);

        Ok(())
    }
    #[tokio::test]
    async fn send_message_without_topic() -> anyhow::Result<()> {
        let (_, command_receiver) = mpsc::channel::<Command>(20);
        let (msg_sender, _) = mpsc::channel::<(Message, ShortPeerId)>(20);

        let mut sw = SwarmManager::build(msg_sender, command_receiver)?;

        for _ in 0..8 {
            let ev = sw.swarm.select_next_some().await;
            sw.handle_event(ev).await;
        }

        let msg = Message::build(Some("Test".to_string()), None).await;
        let res = sw.send_message(msg);

        assert_eq!(
            res.unwrap_err().to_string(),
            SendingError::NoSubscribedTopic.to_string()
        );

        Ok(())
    }

    #[tokio::test]
    async fn send_message_without_connected_peers() -> anyhow::Result<()> {
        let (_, command_receiver) = mpsc::channel::<Command>(20);
        let (msg_sender, _) = mpsc::channel::<(Message, ShortPeerId)>(20);

        let mut sw = SwarmManager::build(msg_sender, command_receiver)?.with_topic("3test");

        for _ in 0..8 {
            let ev = sw.swarm.select_next_some().await;
            sw.handle_event(ev).await;
        }

        let msg = Message::build(Some("Test".to_string()), None).await;
        let res = sw.send_message(msg);

        assert_eq!(
            res.unwrap_err().to_string(),
            SendingError::Other(PublishError::InsufficientPeers).to_string()
        );

        Ok(())
    }

    #[tokio::test]
    async fn subscribe_command() -> anyhow::Result<()> {
        let (command_sender, command_receiver) = mpsc::channel::<Command>(20);
        let (msg_sender, _) = mpsc::channel::<(Message, ShortPeerId)>(20);

        let mut sw = SwarmManager::build(msg_sender, command_receiver)?;

        for _ in 0..8 {
            let ev = sw.swarm.select_next_some().await;
            sw.handle_event(ev).await;
        }

        let (command, _) = Command::new_subscribe("4test".to_string());

        command_sender.send(command).await?;

        let command = sw.command_receiver.recv().await;
        sw.handle_command(command).await;

        assert_eq!(sw.current_topic.unwrap().to_string(), "4test".to_string());

        Ok(())
    }

    #[tokio::test]
    async fn unsubscribe_command() -> anyhow::Result<()> {
        let (command_sender, command_receiver) = mpsc::channel::<Command>(20);
        let (msg_sender, _) = mpsc::channel::<(Message, ShortPeerId)>(20);
        let mut sw = SwarmManager::build(msg_sender, command_receiver)?.with_topic("5test");

        for _ in 0..8 {
            let ev = sw.swarm.select_next_some().await;
            sw.handle_event(ev).await;
        }

        let (command, _) = Command::new_unsubscribe("5test".to_string());

        command_sender.send(command).await?;

        let command = sw.command_receiver.recv().await;
        sw.handle_command(command).await;

        assert!(sw.current_topic.is_none());

        Ok(())
    }
}
