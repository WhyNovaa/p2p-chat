use libp2p::{gossipsub, noise, tcp, yamux, Swarm};
use crate::models::swarm::behaviour::{ChatBehaviour, ChatBehaviourEvent};
use anyhow::Result;
use libp2p::futures::StreamExt;
use libp2p::gossipsub::{IdentTopic, SubscriptionError};
use libp2p::mdns::Event;
use libp2p::swarm::SwarmEvent;
use tokio::io;
use tokio::io::AsyncBufReadExt;
use tokio::sync::mpsc::{Receiver, Sender};
use crate::models::client::command::Command;
use crate::models::swarm::message::{Decode, Encode, Message};

pub struct SwarmManager {
    swarm: Swarm<ChatBehaviour>,
    receiver: Receiver<Command>,
    sender: Sender<Message>,
}

impl SwarmManager {
    pub fn new(sender: Sender<Message>, receiver: Receiver<Command>) -> Result<Self> {
        log::info!("Creating SwarmManager");
        let mut swarm = build_swarm()?;

        let topic = IdentTopic::new("test-net");

        swarm.behaviour_mut()
            .gossipsub
            .subscribe(&topic)?;

        swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

        Ok(Self{
            swarm,
            receiver,
            sender,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        log::info!("Running...");
        let mut stdin = io::BufReader::new(io::stdin()).lines();

        let topic = IdentTopic::new("test-net");

        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => self.handle_event(event).await,
                command = self.receiver.recv() => self.handle_command(command).await,
            }
        }
    }

    pub async fn handle_event(&mut self, event: SwarmEvent<ChatBehaviourEvent>) {
        match event {
            SwarmEvent::Behaviour(ChatBehaviourEvent::Mdns(mdns_event)) => {
                match mdns_event {
                    Event::Discovered(peers) => {
                        for (peer_id, _multiaddr) in peers {
                            log::info!("mDNS discovered a new peer: {peer_id}");
                            self.swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                        }
                    }
                    Event::Expired(peers) => {
                        for (peer_id, _multiaddr) in peers {
                            log::info!("mDNS discover peer has expired: {peer_id}");
                            self.swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                        }
                    }
                }
            },
            SwarmEvent::Behaviour(ChatBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                                                                    propagation_source: peer_id,
                                                                    message_id: id,
                                                                    message,
                                                                })) => {
                let msg = Message::decode(&mut message.data.as_slice());
                log::info!("Got message: '{:?}' with id: {id} from peer: {peer_id}", msg)
            },
            SwarmEvent::NewListenAddr { address, .. } => {
                log::info!("Local node is listening on {address}");
            }
            _ => {}
        }
    }

    async fn handle_command(&mut self, command: Option<Command>) {
        match command {
            Some(Command::SendMessage { response_sender, msg}) => {
                //todo
            }
            Some(Command::Subscribe { response_sender, topic_name,  }) => {
                let ans = match self.subscribe(topic_name).await {
                    Ok(res) => {
                        match res {
                            true => "You have unsubscribed successfully",
                            false => "You haven't been subscribed to this topic",
                        }
                    }
                    Err(_) => "Something went wrong",
                };

                match response_sender.send(ans.to_string()) {
                    Ok(_) => {},
                    Err(e) => log::error!("Response wasn't sent to client: {e}"),
                };
            },
            Some(Command::Unsubscribe { response_sender, topic_name }) => {
                let ans = match self.unsubscribe(topic_name).await {
                    true => "You have unsubscribed successfully",
                    false => "You haven't been subscribed to this topic",
                };

                match response_sender.send(ans.to_string()) {
                    Ok(_) => {},
                    Err(e) => log::error!("Response wasn't sent to client: {e}"),
                };
            },
            None => {}
        }
    }

    pub async fn subscribe(&mut self, topic_name: String) -> Result<bool> {
        log::info!("Subscribing topic...");
        let topic = IdentTopic::new(topic_name);

        let res = self.swarm
            .behaviour_mut()
            .gossipsub
            .subscribe(&topic)?;

        Ok(res)
    }

    pub async fn unsubscribe(&mut self, topic_name: String) -> bool {
        log::info!("Unsubscribing topic...");
        let topic = IdentTopic::new(topic_name);

        let res = self
            .swarm
            .behaviour_mut()
            .gossipsub
            .unsubscribe(&topic);

        res
    }
}

fn build_swarm() -> Result<Swarm<ChatBehaviour>> {
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