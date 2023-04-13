use async_trait::async_trait;
use libp2p::gossipsub::{Behaviour, Config, ConfigBuilder, MessageAuthenticity, ValidationMode};
use libp2p::identity::Keypair;
use libp2p::PeerId;

#[async_trait]
pub trait Gossipsub {
    async fn gossipsub(privacy: MessageAuthenticity, config: Config) -> anyhow::Result<Behaviour> {
        Ok(Behaviour::new(privacy, config).unwrap())
    }

    async fn gossipsub_config(validation_mode: ValidationMode) -> Config {
        ConfigBuilder::default()
            .validation_mode(validation_mode)
            .do_px()
            .build()
            .unwrap()
    }

    /// Message signing is enabled. The author will be the owner of the key and the sequence number
    /// will be a random number.
    async fn message_authenticity_signed(local_key: Keypair) -> MessageAuthenticity {
        MessageAuthenticity::Signed(local_key)
    }

    /// Message signing is disabled.
    ///
    /// The author of the message and the sequence numbers are excluded from the message.
    ///
    /// NOTE: Excluding these fields may make these messages invalid by other nodes who
    /// enforce validation of these fields. See [`ValidationMode`] in the [`Config`]
    /// for how to customise this for rust-libp2p gossipsub.  A custom `message_id`
    /// function will need to be set to prevent all messages from a peer being filtered
    /// as duplicates.
    async fn message_authenticity_author(peer_id: PeerId) -> MessageAuthenticity {
        MessageAuthenticity::Author(peer_id)
    }

    /// Message signing is disabled.
    ///
    /// The author of the message and the sequence numbers are excluded from the message.
    ///
    /// NOTE: Excluding these fields may make these messages invalid by other nodes who
    /// enforce validation of these fields. See [`ValidationMode`] in the [`Config`]
    /// for how to customise this for rust-libp2p gossipsub.  A custom `message_id`
    /// function will need to be set to prevent all messages from a peer being filtered
    /// as duplicates.
    async fn message_authenticity_anonymous() -> MessageAuthenticity {
        MessageAuthenticity::Anonymous
    }

    /// Message signing is disabled.
    /// A random PeerId will be used when publishing each message. The sequence number will be randomized.
    async fn message_authenticity_randomauthor() -> MessageAuthenticity {
        MessageAuthenticity::RandomAuthor
    }
}
