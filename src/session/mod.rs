

use super::ssh;
use super::profile::*;

use anyhow::Ok;
use anyhow::Result;

mod transport;
pub use transport::{AsyncStream, Transport};

mod listener;
pub use listener::Listener;


pub enum Session {
    Ssh(ssh::Session)
}


impl Session {
    #[async_recursion::async_recursion]
    pub async fn new<S: for<'a> super::storage::Storage<'a>>(id: &String, storage: &S) -> Result<Self> {
        let profile = storage.get(id).await?;
        let transport = match profile.transport {
            Some(ref id) => {
                let s = Self::new(id, storage).await?;
                Transport::JumpHost(s)
            },
            None => Transport::Direct
        };
        match profile.protocol {
            Protocol::Ssh(ref c) => {
                let sess = ssh::Session::new(transport, c).await?;
                Ok(Self::Ssh(sess))
            }
        }
    }

    pub async fn local_tunnel(&mut self, host: &str, port: u16) -> Result<AsyncStream> {
        let s = match self {
            Self::Ssh(s) => {
                let ch = s.local_tunnel(host, port).await?;
                AsyncStream::Ssh(ssh::SshStream::from(ch))
            }
        };
        Ok(s)
    }

    pub async fn remote_listener(&mut self, addr: std::net::SocketAddr) -> Result<Listener> {
        let listener = match self {
            Self::Ssh(s) => {
                let l = s.remote_bind(addr).await?;
                Listener::Ssh(l)
            }
        };
        Ok(listener)
    }


}
