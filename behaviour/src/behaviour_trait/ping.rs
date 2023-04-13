use async_trait::async_trait;
use libp2p::ping;

#[async_trait]
pub trait Ping {
    async fn ping() -> ping::Behaviour {
        ping::Behaviour::default()
    }
}
