use std::fmt::{Display, Formatter};
use libp2p::PeerId;

#[derive(Debug, PartialEq, Eq)]
pub struct ShortPeerId(String);

impl ShortPeerId {
    pub fn from(peer_id: &PeerId) -> Self {
        Self(
            format!("{}...", &peer_id.to_string()[..8])
        )
    }
}

impl Display for ShortPeerId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}