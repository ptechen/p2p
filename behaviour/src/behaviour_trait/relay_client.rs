use async_trait::async_trait;
use libp2p::relay::client;

#[async_trait]
pub trait RelayClient {
    async fn relay_client(relay_client: Option<client::Behaviour>) -> client::Behaviour {
        relay_client.expect("missing relay client even though it was enabled")
    }
}
