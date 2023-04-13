use async_trait::async_trait;
use libp2p::{dcutr, PeerId};

#[async_trait]
pub trait Dcutr {
    async fn dcutr(peer_id: PeerId) -> dcutr::Behaviour {
        dcutr::Behaviour::new(peer_id)
    }
}
