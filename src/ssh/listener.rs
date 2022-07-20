
use russh::client;

pub struct RemoteListener {
    ch: client::Channel,
    addr: std::net::SocketAddr
}

impl RemoteListener {
    pub(super) fn new(ch: client::Channel, addr: std::net::SocketAddr) -> Self {
        Self { ch, addr }
    }

    pub async fn close(&mut self) -> Result<(), russh::Error> {
        self.ch.cancel_tcpip_forward(true, self.addr.ip().to_string(), self.addr.port().into()).await
    }

    pub async fn accept(&self) -> Result<(super::SshStream, std::net::SocketAddr), russh::Error> {
        todo!()
    }

    pub fn poll_accept(&self, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(super::SshStream, std::net::SocketAddr), russh::Error>> {
        todo!()
    }
}
