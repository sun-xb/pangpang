
use anyhow::Result;

#[derive(Clone)]
pub struct Manager<S: for<'a> super::storage::Storage<'a> + 'static> {
    storage: S,
}

impl<S: for<'a> super::storage::Storage<'a> + 'static> Manager<S> {
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    pub async fn open_session(&self, id: &String) -> Result<super::session::Session>{
        super::session::Session::new(id, &self.storage).await
    }

    pub async fn open_transport(&self, id: Option<&String>) -> Result<super::session::Transport> {
        let tr = match id {
            Some(id) => {
                let s = self.open_session(id).await?;
                super::session::Transport::JumpHost(s)
            }
            None => super::session::Transport::Direct,
        };
        Ok(tr)
    }

    pub async fn open_http_tunnel(&self, listen_addr: std::net::SocketAddr, local: Option<&String>, remote: Option<String>) -> Result<super::tunnel::Http> {
        let listener = match local {
            Some(id) => {
                let mut s = self.open_session(id).await?;
                s.remote_listener(listen_addr).await?
            }
            None => {
                let l = tokio::net::TcpListener::bind(listen_addr).await?;
                super::session::Listener::Local(l)
            }
        };
        let tunnel_service = super::tunnel::Http::run(listener, super::tunnel::StreamConnector::new(self.clone(), remote));
        Ok(tunnel_service)
    }

    pub async fn open_socks_tunnel(&self, listen_addr: std::net::SocketAddr, local: Option<&String>, remote: Option<String>) -> Result<super::tunnel::Socks> {
        let listener = match local {
            Some(id) => {
                let mut s = self.open_session(id).await?;
                s.remote_listener(listen_addr).await?
            }
            None => {
                let l = tokio::net::TcpListener::bind(listen_addr).await?;
                super::session::Listener::Local(l)
            }
        };
        let tunnel_service = super::tunnel::Socks::run(listener, super::tunnel::StreamConnector::new(self.clone(), remote));
        Ok(tunnel_service)
    }
}


