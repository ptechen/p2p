use async_trait::async_trait;
use libp2p::kad::store::{MemoryStore, MemoryStoreConfig};
use libp2p::kad::{Kademlia, KademliaConfig, ALPHA_VALUE};
use libp2p::multiaddr::Protocol;
use libp2p::{Multiaddr, PeerId};
use std::num::NonZeroUsize;
use std::time::Duration;

#[async_trait]
pub trait Kad {
    async fn kad(
        peer_id: PeerId,
        max_records: Option<usize>,
        max_provided_keys: Option<usize>,
        time_out: u64,
        parallelism: Option<NonZeroUsize>,
        multiaddrs: &Vec<Multiaddr>,
    ) -> Kademlia<MemoryStore> {
        let config = Self::memory_store_config(max_records, max_provided_keys).await;
        let store = MemoryStore::with_config(peer_id, config);
        let kad_config = Self::kad_config(time_out, parallelism).await;
        let mut kademlia = Self::kademlia(peer_id, store, kad_config).await;
        Self::kademlia_add_addresses(&mut kademlia, multiaddrs).await;
        Self::kademlia_bootstrap(&mut kademlia).await;
        kademlia
    }

    async fn memory_store_config(
        max_records: Option<usize>,
        max_provided_keys: Option<usize>,
    ) -> MemoryStoreConfig {
        MemoryStoreConfig {
            max_records: max_records.unwrap_or(1024 * 64),
            max_provided_keys: max_provided_keys.unwrap_or(1024 * 1024),
            ..Default::default()
        }
    }

    async fn memory_store(peer_id: PeerId, config: MemoryStoreConfig) -> MemoryStore {
        MemoryStore::with_config(peer_id, config)
    }

    async fn kad_config(timeout: u64, parallelism: Option<NonZeroUsize>) -> KademliaConfig {
        let mut kad_config = KademliaConfig::default();
        kad_config.set_parallelism(parallelism.unwrap_or(ALPHA_VALUE));
        kad_config.set_query_timeout(Duration::from_secs(timeout));
        kad_config
    }

    async fn kademlia(
        peer_id: PeerId,
        store: MemoryStore,
        config: KademliaConfig,
    ) -> Kademlia<MemoryStore> {
        Kademlia::with_config(peer_id, store, config)
    }

    async fn kademlia_add_addresses(kads: &mut Kademlia<MemoryStore>, multiaddrs: &Vec<Multiaddr>) {
        for multiaddr in multiaddrs {
            let mut addr = multiaddr.to_owned();
            if let Some(Protocol::P2p(mh)) = addr.pop() {
                let peer_id = PeerId::from_multihash(mh).unwrap();
                tracing::warn!("add boot>> {:?}  {:?}", peer_id, addr);
                kads.add_address(&peer_id, addr);
            } else {
                tracing::warn!("Could not parse bootstrap addr {}", multiaddr);
            }
        }
    }

    async fn kademlia_bootstrap(kademlia: &mut Kademlia<MemoryStore>) {
        if let Err(e) = kademlia.bootstrap() {
            tracing::warn!("Kademlia bootstrap failed: {}", e);
        }
    }
}
