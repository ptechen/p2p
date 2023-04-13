use async_trait::async_trait;
use libp2p::identify::{Behaviour, Config};
use libp2p::identity::PublicKey;

#[async_trait]
pub trait Identify {
    async fn identify(config: Config) -> Behaviour {
        Behaviour::new(config)
    }

    async fn identify_config(
        protocol_version: String,
        local_public_key: PublicKey,
        agent_version: String,
        cache: Option<usize>,
    ) -> Config {
        Config::new(protocol_version, local_public_key)
            .with_agent_version(agent_version)
            .with_cache_size(cache.unwrap_or(64 * 1024))
    }
}
