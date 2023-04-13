use async_trait::async_trait;
use libp2p::core::upgrade::{read_length_prefixed, write_length_prefixed};
use libp2p::futures::{AsyncRead, AsyncWrite, AsyncWriteExt};
use libp2p::request_response::{Codec, ProtocolName};
use std::io;

pub const CHAT_PROTOCOL: &str = "/chat/0.1.0";

#[derive(Debug, Clone)]
pub struct ChatProtocol;

impl ProtocolName for ChatProtocol {
    fn protocol_name(&self) -> &[u8] {
        CHAT_PROTOCOL.as_bytes()
    }
}

#[derive(Debug, Clone)]
pub struct ChatRequest(pub Vec<u8>);

impl ChatRequest {
    pub fn data(&self) -> &Vec<u8> {
        self.0.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct ChatResponse(pub Vec<u8>);

#[derive(Debug, Clone)]
pub struct ChatCodec;

#[async_trait]
impl Codec for ChatCodec {
    type Protocol = ChatProtocol;
    type Request = ChatRequest;
    type Response = ChatResponse;

    async fn read_request<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        let data = read_length_prefixed(io, 1024).await?;
        if data.is_empty() {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }
        Ok(ChatRequest(data))
    }

    async fn read_response<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let data = read_length_prefixed(io, 1024).await?;
        if data.is_empty() {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }
        Ok(ChatResponse(data))
    }

    async fn write_request<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
        req: Self::Request,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_length_prefixed(io, req.0).await?;
        io.close().await?;
        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
        res: Self::Response,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_length_prefixed(io, res.0).await?;
        io.close().await?;
        Ok(())
    }
}
