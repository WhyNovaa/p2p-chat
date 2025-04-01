use libp2p::{gossipsub, noise, tcp, yamux, Swarm};
use crate::models::behaviour::{ChatBehaviour, ChatBehaviourEvent};
use anyhow::Result;
use futures::StreamExt;
use libp2p::mdns::Event;
use libp2p::swarm::SwarmEvent;
use tokio::io;
use tokio::io::AsyncBufReadExt;

pub struct SwarmManager {
    swarm: Swarm<ChatBehaviour>
}

impl SwarmManager {
    pub fn new() -> Result<Self> {
        let mut swarm = build_swarm()?;

        let topic = gossipsub::IdentTopic::new("test-net");

        swarm.behaviour_mut()
            .gossipsub
            .subscribe(&topic)?;

        swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

        Ok(Self{ swarm })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut stdin = io::BufReader::new(io::stdin()).lines();

        let topic = gossipsub::IdentTopic::new("test-net");

        loop {
            tokio::select! {
                Ok(Some(line)) = stdin.next_line() => {
                    if let Err(e) = self
                        .swarm.behaviour_mut().gossipsub
                        .publish(topic.clone(), line.as_bytes()) {
                        println!("Publish error: {e:?}");
                    }
                },
                event = self.swarm.select_next_some() => self.handle_event(event).await,
            }
        }
    }

    pub async fn handle_event(&mut self, event: SwarmEvent<ChatBehaviourEvent>) {
        match event {
            SwarmEvent::Behaviour(ChatBehaviourEvent::Mdns(mdns_event)) => {
                match mdns_event {
                    Event::Discovered(peers) => {
                        for (peer_id, _multiaddr) in peers {
                            println!("mDNS discovered a new peer: {peer_id}");
                            self.swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                        }
                    }
                    Event::Expired(peers) => {
                        for (peer_id, _multiaddr) in peers {
                            println!("mDNS discover peer has expired: {peer_id}");
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
                println!("Got message: '{}' with id: {id} from peer: {peer_id}", String::from_utf8_lossy(&message.data))
            },
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Local node is listening on {address}");
            }
            _ => {}
        }
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