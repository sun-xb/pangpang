

use crate::session::Listener;
use crate::session::AsyncStream;
use crate::storage::Storage;

use std::sync::Arc;
use tokio::sync::Notify;

pub struct Server {
    notify: Arc<Notify>
}

impl Server {
    pub fn run<S: for<'a> Storage<'a> + 'static>(mut listener: Listener, connector: super::StreamConnector<S>) -> Self {
        let notify = Notify::default();
        let notify = Arc::new(notify);
        let shutdown = notify.clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    r = listener.accept() => {
                        if let Err(e) = r {
                            log::error!("exited with error: {:?}", e);
                            break;
                        }
                        let (socket, addr) = r.unwrap();
                        log::debug!("connection from {}", addr.to_string());
                        let handle_shutdown = shutdown.clone();
                        let c = connector.clone();
                        tokio::spawn(async move {
                            tokio::select! {
                                r = Self::handle(socket, c) => {
                                    if let Err(e) = r {
                                        log::error!("socks forward error: {:?}", e);
                                    }
                                }
                                _ = handle_shutdown.notified() => (),
                            }
                        });
                    }
                    _ = shutdown.notified() => break,
                }
            }
        });
        Self{ notify }
    }

    async fn handle<S: for<'a> Storage<'a> + 'static>(mut s: AsyncStream, connector: super::StreamConnector<S>) -> anyhow::Result<()> {
        let handshake_req = socks5_proto::HandshakeRequest::read_from(&mut s).await?;
        if handshake_req.methods.contains(&socks5_proto::HandshakeMethod::None) {
            let resp = socks5_proto::HandshakeResponse::new(socks5_proto::HandshakeMethod::None);
            resp.write_to(&mut s).await?;
        } else {
            let resp = socks5_proto::HandshakeResponse::new(socks5_proto::HandshakeMethod::Unacceptable);
            resp.write_to(&mut s).await?;
            return Err(anyhow::anyhow!("no available handshake method provided by client"));
        }
        let req = match socks5_proto::Request::read_from(&mut s).await {
            Ok(req) => req,
            Err(e) => {
                let resp = socks5_proto::Response::new(socks5_proto::Reply::GeneralFailure, socks5_proto::Address::unspecified());
                resp.write_to(&mut s).await?;
                return Err(anyhow::anyhow!(e));
            }
        };
        match req.command {
            socks5_proto::Command::Connect => {
                let conn_result = match req.address {
                    socks5_proto::Address::DomainAddress(host, port) => connector.connect(host.as_str(), port).await,
                    socks5_proto::Address::SocketAddress(addr) => connector.connect(addr.ip().to_string().as_str(), addr.port().into()).await,
                };
                let mut conn = match conn_result {
                    Ok(s) => s,
                    Err(e) => {
                        let resp = socks5_proto::Response::new(socks5_proto::Reply::NetworkUnreachable, socks5_proto::Address::unspecified());
                        resp.write_to(&mut s).await?;
                        return Err(anyhow::anyhow!(e));
                    }
                };
                socks5_proto::Response::new(socks5_proto::Reply::Succeeded, socks5_proto::Address::unspecified()).write_to(&mut s).await?;
                tokio::io::copy_bidirectional(&mut s, &mut conn).await?;
            }
            _ => {
                socks5_proto::Response::new(socks5_proto::Reply::CommandNotSupported, socks5_proto::Address::unspecified()).write_to(&mut s).await?;
            }
        }
        Ok(())
    }

    pub fn shutdown(&self) {
        self.notify.notify_waiters();
    }
}