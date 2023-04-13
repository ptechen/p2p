use async_trait::async_trait;
use libp2p::{autonat, PeerId};
use std::time::Duration;

#[async_trait]
pub trait Autonat {
    async fn autonat(peer_id: PeerId, config: autonat::Config) -> autonat::Behaviour {
        autonat::Behaviour::new(peer_id, config)
    }

    async fn autonat_config(
        use_connected: bool,
        boot_delay: u64,
        refresh_interval: u64,
        retry_interval: u64,
    ) -> autonat::Config {
        autonat::Config {
            use_connected,
            boot_delay: Duration::from_secs(boot_delay),
            refresh_interval: Duration::from_secs(refresh_interval),
            retry_interval: Duration::from_secs(retry_interval),
            ..Default::default()
        }
    }
}
