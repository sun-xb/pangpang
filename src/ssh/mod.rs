use russh::client;



mod handler;
mod profile;
mod stream;
mod listener;
mod pty;

pub use profile::Profile;
pub use stream::SSHStream;
pub use listener::RemoteListener;
pub use pty::SSHPty;

pub struct Session {
    handle: client::Handle<handler::Handler>
}

impl Session {
    pub async fn new(mut transport: super::session::Transport, c: &Profile) -> anyhow::Result<Self> {
        let config = std::sync::Arc::new(client::Config::default());
        let stream = transport.connect(c.host.as_str(), c.port).await?;
        let mut handle = client::connect_stream(config, stream, handler::Handler).await?;
        anyhow::ensure!(true == handle.authenticate_password(c.username.as_str(), c.password.as_str()).await?, "ssh authenticate failed");
        Ok(Self{ handle })
    }

    pub async fn local_tunnel(&mut self, host: &str, port: u16) -> Result<russh::Channel<client::Msg>, russh::Error> {
        self.handle.channel_open_direct_tcpip(host, port.into(), "0.0.0.0", 0).await
    }

    pub async fn remote_bind(&mut self, addr: std::net::SocketAddr) -> Result<RemoteListener, russh::Error> {
        if self.handle.tcpip_forward(addr.ip().to_string(), addr.port() as u32).await? {
            Ok(RemoteListener::new(addr))
        } else {
            Err(russh::Error::HUP)
        }
    }
}