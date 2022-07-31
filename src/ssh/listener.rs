
use russh::client;

pub struct RemoteListener {
    addr: std::net::SocketAddr
}

impl RemoteListener {
    pub(super) fn new(addr: std::net::SocketAddr) -> Self {
        Self { addr }
    }

    pub async fn close(&mut self) -> Result<(), russh::Error> {
        //self.ch.cancel_tcpip_forward(true, self.addr.ip().to_string(), self.addr.port().into()).await
        
        Ok(())
    }

    pub async fn accept(&self) -> Result<(super::SSHStream, std::net::SocketAddr), russh::Error> {
        todo!()
    }

    pub fn poll_accept(&self, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(super::SSHStream, std::net::SocketAddr), russh::Error>> {
        todo!()
    }
}
