use async_std::io;
use clap::Parser;
use futures::stream::StreamExt;
use libp2p::{
    core::multiaddr::Protocol,
    core::upgrade,
    core::{Multiaddr, Transport},
    identify, identity,
    identity::PeerId,
    kad::store::MemoryStore,
    noise, ping, relay, request_response,
    swarm::{NetworkBehaviour, SwarmBuilder, SwarmEvent},
    tcp,
};
use std::iter;
use std::{io::BufRead, time::Duration};

use futures::{future::Either, prelude::*, select};
use libp2p::core::transport::OrTransport;
use libp2p::request_response::{Config, ProtocolSupport};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
// use tokio::io::AsyncBufReadExt;
use codec::chat::ChatResponse;
use codec::chat::{ChatCodec, ChatProtocol, ChatRequest, CHAT_PROTOCOL};

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let opt = Opt::parse();
    println!("opt: {opt:?}");

    let relay_server = generate_ed25519(opt.server_secret_key_seed);
    let relay_peer_id = PeerId::from(relay_server.public());

    // Create a static known PeerId based on given secret
    let client: identity::Keypair = generate_ed25519(opt.client_secret_key_seed);
    let local_peer_id = PeerId::from(client.public());
    println!("Local peer id: {local_peer_id:?}");

    // Create a static known PeerId based on given secret
    let receive: identity::Keypair = generate_ed25519(opt.receive_secret_key_seed);
    let receive_peer_id = PeerId::from(receive.public());

    let (transport, behaviour) = relay::client::new(local_peer_id);
    let tcp_transport = OrTransport::new(transport, tcp::async_io::Transport::default());

    let transport = tcp_transport
        .upgrade(upgrade::Version::V1)
        .authenticate(
            noise::NoiseAuthenticated::xx(&client)
                .expect("Signing libp2p-noise static DH keypair failed."),
        )
        .multiplex(libp2p::yamux::YamuxConfig::default())
        .timeout(std::time::Duration::from_secs(20))
        .boxed();

    let protocols = iter::once((ChatProtocol, ProtocolSupport::Full));
    let behaviour = Behaviour {
        relay: behaviour,
        ping: ping::Behaviour::new(ping::Config::default()),
        identify: identify::Behaviour::new(identify::Config::new(
            "/TODO/0.0.1".to_string(),
            client.public(),
        )),
        chat: request_response::Behaviour::new(ChatCodec, protocols.clone(), Config::default()),
        keep_alive: libp2p::swarm::keep_alive::Behaviour::default(),
        kad: libp2p::kad::Kademlia::new(local_peer_id, MemoryStore::new(local_peer_id)),
    };

    let mut swarm =
        SwarmBuilder::with_async_std_executor(transport, behaviour, local_peer_id).build();

    // Listen on all interfaces
    let client_addr = Multiaddr::empty()
        .with(match opt.use_ipv6 {
            Some(true) => Protocol::from(Ipv6Addr::UNSPECIFIED),
            _ => Protocol::from(Ipv4Addr::from_str("127.0.0.1")?),
        })
        .with(Protocol::Tcp(opt.port))
        .with(Protocol::P2p(relay_peer_id.into()))
        .with(Protocol::P2pCircuit);
    println!("{}", client_addr.to_string());
    swarm.listen_on(client_addr.clone())?;

    let receive_addr = client_addr
        .clone()
        .with(Protocol::P2p(receive_peer_id.into()));
    println!("{}", receive_addr.to_string());
    // Read full lines from stdin
    let mut stdin = async_std::io::BufReader::new(async_std::io::stdin())
        .lines()
        .fuse();
    swarm
        .behaviour_mut()
        .chat
        .add_address(&receive_peer_id, receive_addr.clone());

    //SwarmEvent::OutgoingConnectionError { peer_id, error } => {println!("{peer_id:?} {error:?}");}
    loop {
        select! {
            line = stdin.select_next_some() => {
                let req_id = swarm.behaviour_mut().chat.send_request(&receive_peer_id, ChatRequest(line.expect("Stdin not to close").as_bytes().to_vec()));
                // let req_id = swarm.behaviour_mut().chat.send_request(&PeerId::from_str("12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X").unwrap(), ChatRequest(line.expect("Stdin not to close").as_bytes().to_vec()));
                println!("req id: {req_id:?}");
            },
            event = swarm.select_next_some() => match event {
                SwarmEvent::OutgoingConnectionError { peer_id, error } => {println!("OutgoingConnectionError: {peer_id:?} {error:?}");}
                SwarmEvent::ListenerError { listener_id, error } => {println!("ListenerError: {listener_id:?} {error:?}");}
                SwarmEvent::ListenerClosed { listener_id, addresses, reason }  => {println!("ListenerClosed: {listener_id:?} {addresses:?} {reason:?}");}
                SwarmEvent::ExpiredListenAddr { listener_id, address }  => {println!("ExpiredListenAddr: {listener_id:?} {address:?}");}
                SwarmEvent::BannedPeer { peer_id, endpoint } => {println!("{peer_id:?} {endpoint:?}");}
                SwarmEvent::IncomingConnectionError { local_addr, send_back_addr, error  } => {println!("IncomingConnectionError: {local_addr:?} {send_back_addr:?} {error:?}");}
                SwarmEvent::IncomingConnection { local_addr , send_back_addr } => {println!("IncomingConnection: {local_addr:?} {send_back_addr:?}");}
                SwarmEvent::Dialing(peer_id) => {
                    println!("Dialing: {peer_id}");
                }
                SwarmEvent::ConnectionEstablished { peer_id, .. } if peer_id == relay_peer_id => { println!("ConnectionEstablished: {peer_id}"); }
                SwarmEvent::ConnectionClosed { peer_id, ..} => {
                    println!("ConnectionClosed: {peer_id}");
                }
                SwarmEvent::OutgoingConnectionError { peer_id, error } if peer_id == Some(relay_peer_id) => {
                    println!("OutgoingConnectionError: {peer_id:?} err: {error:?}");
                }
                SwarmEvent::NewListenAddr { address, .. } => {
                    // swarm.listen_on(address.clone())?;
                    println!("Listening on {address:?}");
                }
                SwarmEvent::Behaviour(event) => {
                    match event {
                        BehaviourEvent::Relay(relay::client::Event::ReservationReqAccepted {
                            relay_peer_id, renewal, ..
                        }) => {
                            println!("ReservationReqAccepted{renewal}");
                            if renewal {
                                // break;
                            }
                        },
                        BehaviourEvent::Ping(e) => {
                            // println!("Ping: {e:?}");
                        }
                        BehaviourEvent::Chat(e) => {
                            match e {
                                request_response::Event::Message { peer, message } => {
                                    println!("Event::Message: {peer}");
                                    match message {
                                        request_response::Message::Response { request_id, response } => {
                                            println!("Message::Response: {request_id:?} {response:?}")
                                        }
                                        request_response::Message::Request { request_id, request, channel } => {
                                            println!("Message::Request: {request_id:?} {request:?}");
                                            swarm.behaviour_mut().chat.send_response(channel, ChatResponse(request.data().clone())).unwrap();
                                        }
                                    }
                                }
                                request_response::Event::ResponseSent { peer, request_id } => {
                                    println!("ResponseSent: {peer:?} {request_id:?}");
                                }
                                request_response::Event::OutboundFailure { peer, request_id, error } => {
                                    println!("OutboundFailure: {peer:?} {request_id:?} error: {error:?}");
                                }
                                request_response::Event::InboundFailure { peer, request_id, error } => {
                                    println!("InboundFailure: {peer:?} {request_id:?} error: {error:?}");
                                }
                            }
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

                        _ => {}
                    }
                }
                _ => {}
                // SwarmEvent::Behaviour(ClientEvent::Ping(_)) => {}
               // _ => { println!("run") }
                // e => panic!("{e:?}"),

            }
        }
    }
    Ok(())
}

#[derive(NetworkBehaviour)]
struct Behaviour {
    relay: relay::client::Behaviour,
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
    server_secret_key_seed: u8,

    #[clap(long)]
    client_secret_key_seed: u8,

    #[clap(long)]
    receive_secret_key_seed: u8,

    /// The port used to listen on all interfaces
    #[clap(long)]
    port: u16,
}
