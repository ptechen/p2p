use async_trait::async_trait;
use libp2p::swarm::keep_alive;

#[async_trait]
pub trait KeepAlive {
    async fn keep_alive() -> keep_alive::Behaviour {
        keep_alive::Behaviour::default()
    }
}
