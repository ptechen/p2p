// use libp2p::{kad, PeerId, ping, relay, request_response, Swarm, Transport, tcp, noise};
// use libp2p::swarm::{keep_alive, NetworkBehaviour, SwarmBuilder};
// use libp2p::{autonat, dcutr, identify};
// use libp2p::core::muxing::StreamMuxerBox;
// use libp2p::core::transport::{Boxed, MemoryTransport, OrTransport};
// use libp2p::core::upgrade;
// use libp2p::identity::{Keypair, PublicKey};
// use libp2p::plaintext::PlainText2Config;
// use libp2p::swarm::behaviour::toggle::Toggle;
// use codec::chat::{ChatCodec, ChatRequest, ChatResponse};
// use futures::io::{AsyncRead, AsyncWrite};
// use futures::future::Either;
// use void;
// use crate::behaviour_trait::autonat::Autonat;
// use crate::behaviour_trait::chat::Chat;
// use crate::behaviour_trait::dcutr::Dcutr;
// use crate::behaviour_trait::identify::Identify;
// use crate::behaviour_trait::relay_server::RelayServer;
//
//
//
// #[derive(NetworkBehaviour)]
// #[behaviour(out_event = "Event")]
// pub struct Behaviour {
//     pub ping: ping::Behaviour,
//     pub identify: identify::Behaviour,
//     pub keep_alive: keep_alive::Behaviour,
//     // pub auto_nat: autonat::Behaviour,
//     pub dcutr: dcutr::Behaviour,
//     pub chat: request_response::Behaviour<ChatCodec>,
//     pub relay_server: Toggle<relay::Behaviour>,
//     pub relay_client: Toggle<relay::client::Behaviour>,
// }
//
//
// impl Identify for Behaviour{}
//
// impl Autonat for Behaviour {}
//
// impl Dcutr for Behaviour {}
//
// impl RelayServer for Behaviour {}
//
// impl Chat for Behaviour{}
//
// impl Behaviour {
//     pub async fn new_relay_server(keypair: &Keypair) -> anyhow::Result<Swarm<Self>> {
//         let local_public_key = keypair.public();
//         let peer_id = local_public_key.to_peer_id();
//         let tcp_transport = tcp::tokio::Transport::default();
//         let transport = Self::upgrade_transport(tcp_transport, local_public_key.clone());
//         let behaviour = Self{
//             ping: ping::Behaviour::default(),
//             identify: Self::identify(Self::identify_config("/identify/0.1.0".to_string(), local_public_key, "relay".to_string(), Some(64 * 1024)).await).await,
//             keep_alive: Default::default(),
//             // auto_nat: Self::autonat(peer_id, Self::autonat_config(false, 0, 5, 5).await).await,
//             dcutr: Self::dcutr(peer_id).await,
//             chat: Self::chat(&vec![]).await?.unwrap(),
//             relay_server: Some(Self::replay_server(peer_id, Default::default()).await).into(),
//             relay_client: Toggle::from(None),
//         };
//         Ok(SwarmBuilder::with_tokio_executor(transport, behaviour, peer_id).build())
//     }
//
//     pub async fn new_relay_client(keypair: &Keypair) -> anyhow::Result<Swarm<Behaviour>> {
//         let local_public_key = keypair.public();
//         let peer_id = local_public_key.to_peer_id();
//
//         let (relay_transport, relay_client) = relay::client::new(peer_id);
//         let data = OrTransport::new(relay_transport, tcp::tokio::Transport::default()).boxed();
//         let transport = Self::upgrade_transport(
//             data,
//             keypair.clone(),
//         );
//
//         let b = Self{
//             ping: ping::Behaviour::default(),
//             identify: Self::identify(Self::identify_config("/identify/0.1.0".to_string(), local_public_key, "relay".to_string(), Some(64 * 1024)).await).await,
//             keep_alive: Default::default(),
//             // auto_nat: Self::autonat(peer_id, Self::autonat_config(true, 0, 5, 5).await).await,
//             dcutr: Self::dcutr(peer_id).await,
//             chat: Self::chat(&vec!["/ip4/127.0.0.1/tcp/9999".parse()?]).await?.unwrap(),
//             relay_server: Toggle::from(None),
//             relay_client: Some(relay_client).into(),
//         };
//
//         Ok(SwarmBuilder::with_tokio_executor(
//             transport,
//             b,
//             peer_id,
//         )
//             .build())
//     }
//
//     fn upgrade_transport(
//         transport: tcp::tokio::Transport,
//         local_key: Keypair,
//     ) -> Boxed<(PeerId, StreamMuxerBox)>
//         where
//             StreamSink: AsyncRead + AsyncWrite + Send + Unpin + 'static,
//     {
//         transport
//             .upgrade(upgrade::Version::V1)
//             .authenticate(noise::NoiseAuthenticated::xx(&local_key))
//             .multiplex(libp2p::yamux::YamuxConfig::default())
//             .boxed()
//     }
// }
//
// #[derive(Debug)]
// pub enum Event {
//     Ping(ping::Event),
//     Identify(identify::Event),
//     Kademlia(kad::KademliaEvent),
//     // Mdns(mdns::Event),
//     Chat(request_response::Event<ChatRequest, ChatResponse>),
//     Autonat(autonat::Event),
//     Relay(relay::Event),
//     RelayClient(relay::client::Event),
//     Dcutr(dcutr::Event),
//     // Gossipsub(gossipsub::Event),
// }
//
// impl From<ping::Event> for Event {
//     fn from(event: ping::Event) -> Self {
//         Event::Ping(event)
//     }
// }
//
// impl From<identify::Event> for Event {
//     fn from(event: identify::Event) -> Self {
//         Event::Identify(event)
//     }
// }
//
// impl From<kad::KademliaEvent> for Event {
//     fn from(event: kad::KademliaEvent) -> Self {
//         Event::Kademlia(event)
//     }
// }
//
// // impl From<mdns::Event> for Event {
// //     fn from(event: mdns::Event) -> Self {
// //         Event::Mdns(event)
// //     }
// // }
//
// // impl From<gossipsub::Event> for Event {
// //     fn from(event: gossipsub::Event) -> Self {
// //         Event::Gossipsub(event)
// //     }
// // }
//
// // impl From<autonat::Event> for Event {
// //     fn from(event: autonat::Event) -> Self {
// //         Event::Autonat(event)
// //     }
// // }
//
// impl From<relay::Event> for Event {
//     fn from(event: relay::Event) -> Self {
//         Event::Relay(event)
//     }
// }
//
// impl From<relay::client::Event> for Event {
//     fn from(event: relay::client::Event) -> Self {
//         Event::RelayClient(event)
//     }
// }
//
// impl From<dcutr::Event> for Event {
//     fn from(event: dcutr::Event) -> Self {
//         Event::Dcutr(event)
//     }
// }
//
// impl From<request_response::Event<ChatRequest, ChatResponse>> for Event {
//     fn from(event: request_response::Event<ChatRequest, ChatResponse>) -> Self {
//         Event::Chat(event)
//     }
// }
//
// impl From<void::Void> for Event {
//     fn from(e: void::Void) -> Self {
//         void::unreachable(e)
//     }
// }
