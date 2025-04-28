use anyhow::Result;
use libp2p::identity::Keypair;
use libp2p::swarm::NetworkBehaviour;
use libp2p::{gossipsub, mdns};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::Duration;

#[derive(NetworkBehaviour)]
pub struct ChatBehaviour {
    pub mdns: mdns::tokio::Behaviour,
    pub gossipsub: gossipsub::Behaviour,
}

impl ChatBehaviour {
    pub fn build(key: &Keypair) -> Result<Self> {
        let message_id_fn = |message: &gossipsub::Message| {
            let sequence_number = message.sequence_number.unwrap_or(0);
            let peer_id_as_base58 = message
                .source
                .as_ref()
                .map_or("".to_string(), |s| s.to_base58());

            let unique_id = format!("{}-{}", sequence_number, peer_id_as_base58);

            let mut s = DefaultHasher::new();
            unique_id.hash(&mut s);

            gossipsub::MessageId::from(s.finish().to_string())
        };

        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .message_id_fn(message_id_fn)
            .build()
            .map_err(|msg| std::io::Error::new(std::io::ErrorKind::Other, msg))?;

        let gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(key.clone()),
            gossipsub_config,
        )
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())?;
        Ok(ChatBehaviour { gossipsub, mdns })
    }
}
