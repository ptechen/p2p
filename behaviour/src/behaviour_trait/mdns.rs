use async_trait::async_trait;
use libp2p::mdns::Config;
use libp2p::{mdns, PeerId};

#[async_trait]
pub trait Mdns {
    async fn mdns(config: Config, peer_id: PeerId) -> anyhow::Result<mdns::tokio::Behaviour> {
        Ok(mdns::tokio::Behaviour::new(config, peer_id)?)
    }

    async fn mdns_default(peer_id: PeerId) -> anyhow::Result<mdns::tokio::Behaviour> {
        Ok(mdns::tokio::Behaviour::new(Default::default(), peer_id)?)
    }
}
