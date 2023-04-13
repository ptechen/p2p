use clap::Parser;
use codec::chat::{ChatCodec, ChatProtocol, ChatRequest, ChatResponse};
use futures::stream::StreamExt;
use libp2p::kad::store::MemoryStore;
use libp2p::request_response::{Config, Event, Message, ProtocolSupport};
use libp2p::swarm::AddressScore;
use libp2p::{
    core::multiaddr::Protocol,
    core::upgrade,
    core::{Multiaddr, Transport},
    identify, identity,
    identity::PeerId,
    noise, ping, relay, request_response,
    swarm::{NetworkBehaviour, SwarmBuilder, SwarmEvent},
    tcp,
};
use std::iter;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let opt = Opt::parse();
    println!("opt: {opt:?}");

    // Create a static known PeerId based on given secret
    let local_key: identity::Keypair = generate_ed25519(opt.secret_key_seed);
    let relay_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {relay_peer_id:?}");

    let tcp_transport = tcp::tokio::Transport::default();

    let transport = tcp_transport
        .upgrade(upgrade::Version::V1)
        .authenticate(
            noise::NoiseAuthenticated::xx(&local_key)
                .expect("Signing libp2p-noise static DH keypair failed."),
        )
        .multiplex(libp2p::yamux::YamuxConfig::default())
        .timeout(std::time::Duration::from_secs(20))
        .boxed();

    let protocols = iter::once((ChatProtocol, ProtocolSupport::Full));
    let behaviour = Behaviour {
        relay: relay::Behaviour::new(relay_peer_id, relay::Config::default()),
        ping: ping::Behaviour::new(ping::Config::new()),
        identify: identify::Behaviour::new(identify::Config::new(
            "/TODO/0.0.1".to_string(),
            local_key.public(),
        )),
        chat: request_response::Behaviour::new(ChatCodec, protocols.clone(), Config::default()),
        keep_alive: libp2p::swarm::keep_alive::Behaviour::default(),
        kad: libp2p::kad::Kademlia::new(relay_peer_id, MemoryStore::new(relay_peer_id)),
    };

    let client_addr = Multiaddr::empty().with(Protocol::P2pCircuit);

    let mut swarm = SwarmBuilder::with_tokio_executor(transport, behaviour, relay_peer_id).build();

    // Listen on all interfaces
    let listen_addr = Multiaddr::empty()
        .with(match opt.use_ipv6 {
            Some(true) => Protocol::from(Ipv6Addr::UNSPECIFIED),
            _ => Protocol::from(Ipv4Addr::UNSPECIFIED),
        })
        .with(Protocol::Tcp(opt.port));
    swarm.listen_on(listen_addr.clone())?;
    swarm.add_external_address(listen_addr.clone(), AddressScore::Infinite);

    // swarm.behaviour_mut().chat.add_address(
    //     &PeerId::from_str("12D3KooWQYhTNQdmr3ArTeUHRYzFg94BKyTkoWBDWez9kSCVe2Xo").unwrap(),
    //     receive_addr.clone(),
    // );

    loop {
        match swarm.next().await.expect("Infinite Stream.") {
            SwarmEvent::Behaviour(event) => match event {
                BehaviourEvent::Ping(e) => {
                    // println!("Ping: {e:?}")
                }
                BehaviourEvent::Kad(event) => match event {
                    libp2p::kad::KademliaEvent::InboundRequest { request } => {
                        println!("InboundRequest: {request:?}");
                    }
                    libp2p::kad::KademliaEvent::OutboundQueryProgressed {
                        id,
                        result,
                        stats,
                        step,
                    } => {
                        println!("OutboundQueryProgressed: {id:?} {result:?} {stats:?} {step:?}");
                    }
                    libp2p::kad::KademliaEvent::RoutingUpdated {
                        peer,
                        is_new_peer,
                        addresses,
                        bucket_range,
                        old_peer,
                    } => {
                        println!("RoutingUpdated: {peer:?} {is_new_peer:?} {addresses:?} {bucket_range:?} {old_peer:?}");
                    }
                    libp2p::kad::KademliaEvent::UnroutablePeer { peer } => {
                        println!("UnroutablePeer: {peer:?}");
                    }
                    libp2p::kad::KademliaEvent::RoutablePeer { peer, address } => {
                        println!("RoutablePeer: {peer:?} {address:?}");
                    }
                    libp2p::kad::KademliaEvent::PendingRoutablePeer { peer, address } => {
                        println!("PendingRoutablePeer: {peer:?} {address:?}");
                    }
                },
                BehaviourEvent::Chat(e) => match e {
                    request_response::Event::ResponseSent { peer, request_id } => {
                        println!("ResponseSent: {peer:?} {request_id:?}");
                    }
                    request_response::Event::Message { peer, message } => match message {
                        Message::Response {
                            request_id,
                            response,
                        } => {
                            println!("{request_id:?} {response:?}")
                        }
                        Message::Request {
                            request_id,
                            request,
                            channel,
                        } => {
                            swarm
                                .behaviour_mut()
                                .chat
                                .send_response(channel, ChatResponse("ok".as_bytes().to_vec()))
                                .unwrap();
                        }
                    },
                    request_response::Event::OutboundFailure {
                        peer,
                        request_id,
                        error,
                    } => {
                        println!("OutboundFailure: {peer:?} {request_id:?} error: {error:?}");
                    }
                    request_response::Event::InboundFailure {
                        peer,
                        request_id,
                        error,
                    } => {
                        println!("InboundFailure: {peer:?} {request_id:?} error: {error:?}");
                    }
                },
                BehaviourEvent::Relay(event) => {
                    println!("Relay: {event:?}");
                }
                _ => {}
            },
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on {address:?}");
            }
            _ => {}
        }
    }
}

#[derive(NetworkBehaviour)]
struct Behaviour {
    relay: relay::Behaviour,
    ping: ping::Behaviour,
    identify: identify::Behaviour,
    chat: request_response::Behaviour<ChatCodec>,
    keep_alive: libp2p::swarm::keep_alive::Behaviour,
    kad: libp2p::kad::Kademlia<MemoryStore>,
}

fn generate_ed25519(secret_key_seed: u8) -> identity::Keypair {
    let mut bytes = [0u8; 32];
    bytes[0] = secret_key_seed;

    identity::Keypair::ed25519_from_bytes(bytes).expect("only errors on wrong length")
}

#[derive(Debug, Parser)]
#[clap(name = "libp2p relay")]
struct Opt {
    /// Determine if the relay listen on ipv6 or ipv4 loopback address. the default is ipv4
    #[clap(long)]
    use_ipv6: Option<bool>,

    /// Fixed value to generate deterministic peer id
    #[clap(long)]
    secret_key_seed: u8,

    /// The port used to listen on all interfaces
    #[clap(long)]
    port: u16,
}
