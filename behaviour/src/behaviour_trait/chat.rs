use async_trait::async_trait;
use codec::chat;
use codec::chat::ChatCodec;
use libp2p::multiaddr::Protocol;
use libp2p::request_response::{Behaviour, ProtocolSupport};
use libp2p::{Multiaddr, PeerId};
use std::iter;

#[async_trait]
pub trait Chat {
    async fn chat(
        bootstrap_peers: &Vec<Multiaddr>,
    ) -> anyhow::Result<Option<Behaviour<ChatCodec>>> {
        let cfg = libp2p::request_response::Config::default();
        let protocols = iter::once((chat::ChatProtocol, ProtocolSupport::Full));
        let mut chat = Some(Behaviour::new(chat::ChatCodec, protocols, cfg));
        chat.as_mut().map(|rq| {
            for multiaddr in bootstrap_peers {
                let mut addr = multiaddr.to_owned();
                if let Some(Protocol::P2p(mh)) = addr.pop() {
                    let peer_id = PeerId::from_multihash(mh).unwrap();
                    tracing::info!("add boot to chat>> {:?}  {:?}", peer_id, addr);
                    rq.add_address(&peer_id, addr);
                } else {
                    tracing::info!("Could not parse bootstrap addr {}", multiaddr);
                }
            }
        });
        Ok(chat)
    }
}
