use async_trait::async_trait;
use libp2p::relay::Config;
use libp2p::PeerId;

#[async_trait]
pub trait RelayServer {
    async fn replay_server(peer_id: PeerId, config: Config) -> libp2p::relay::Behaviour {
        libp2p::relay::Behaviour::new(peer_id, config)
    }

    async fn replay_server_config() -> Config {
        Config::default()
    }
}
