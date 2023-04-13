use crate::behaviour_trait::autonat::Autonat;
use crate::behaviour_trait::dcutr::Dcutr;
use crate::behaviour_trait::gossipsub::Gossipsub;
use crate::behaviour_trait::identify::Identify;
use crate::behaviour_trait::kad::Kad;
use crate::behaviour_trait::keep_alive::KeepAlive;
use crate::behaviour_trait::mdns::Mdns;
use crate::behaviour_trait::ping::Ping;
use crate::behaviour_trait::relay_client::RelayClient;
use crate::behaviour_trait::relay_server::RelayServer;

pub mod autonat;
pub mod chat;
pub mod dcutr;
pub mod gossipsub;
pub mod identify;
pub mod kad;
pub mod keep_alive;
pub mod mdns;
pub mod ping;
pub mod relay_client;
pub mod relay_server;

pub trait AllTrait:
    Ping + KeepAlive + Identify + Mdns + Autonat + Dcutr + Kad + Gossipsub + RelayClient + RelayServer
{
}
