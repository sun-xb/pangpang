

use std::future::Future;

use crate::session::*;

pub struct Server {
    notify: std::sync::Arc<tokio::sync::Notify>,
}

impl Server {
    pub fn shutdown(&mut self) {
        self.notify.notify_one();
    }

    pub fn run<S: for<'a> super::Storage<'a> + 'static>(listener: Listener, stream_connector: super::StreamConnector<S>) -> Self {
        let connector = TunnelConnector::new(stream_connector.clone());
        let http_client = hyper::Client::builder().build::<_, hyper::Body>(connector);
        let make_svc = hyper::service::make_service_fn(move |_| {
            let client = http_client.clone();
            let connector = stream_connector.clone();
            let svc = hyper::service::service_fn(move |r| Self::forward(r, client.clone(), connector.clone()) );
            std::future::ready(Ok::<_, anyhow::Error>(svc))
        });
        let notify = std::sync::Arc::new(tokio::sync::Notify::default());
        let shutdown = notify.clone();
        tokio::spawn(async move {
            let result = hyper::Server::builder(listener)
                .serve(make_svc)
                .with_graceful_shutdown(shutdown.notified())
                .await;
            if let Err(e) = result {
                log::error!("http tunnel error: {:?}", e);
            }
        });
        Self{ notify }
    }

    async fn forward<S: for<'a> super::Storage<'a> + 'static>(r: hyper::Request<hyper::Body>, http_client: hyper::Client<TunnelConnector<S>>, connector: super::StreamConnector<S>) -> anyhow::Result<hyper::Response<hyper::Body>> {
        log::debug!("request: {:?}", r);
        let resp = match r.method() {
            &hyper::Method::CONNECT => {
                tokio::spawn(async move {
                    if let Err(e) = Self::forward_connect(r, connector).await {
                        log::error!("connect forward error: {:?}", e);
                    }
                });
                hyper::Response::new(hyper::Body::empty())
            }
            _ => http_client.request(r).await?
        };
        Ok(resp)
    }

    async fn forward_connect<S: for<'a> super::Storage<'a> + 'static>(r: hyper::Request<hyper::Body>, connector: super::StreamConnector<S>) -> anyhow::Result<()> {
        let (host, port) = r.uri().get_addr()?;
        let mut stream = connector.connect(host, port).await?;
        let mut upgrade = hyper::upgrade::on(r).await?;
        if let Err(e) = tokio::io::copy_bidirectional(&mut upgrade, &mut stream).await {
            if std::io::ErrorKind::UnexpectedEof != e.kind() {
                return Err(anyhow::Error::from(e));
            }
        }
        Ok(())
    }

}


#[derive(Clone)]
struct TunnelConnector<S: for<'a> super::Storage<'a> + 'static> {
    connector: super::StreamConnector<S>
}
impl<S: for<'a> super::Storage<'a> + 'static> TunnelConnector<S> {
    pub fn new(connector: super::StreamConnector<S>) -> Self {
        Self { connector }
    }
}

impl<S: for<'a> super::Storage<'a> + 'static> hyper::service::Service<hyper::Uri> for TunnelConnector<S> {
    type Response = AsyncStream;
    type Error = anyhow::Error;
    type Future = std::pin::Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: hyper::Uri) -> Self::Future {
        let connector = self.connector.clone();
        let f = async move {
            let (host, port) = req.get_addr()?;
            let s = connector.connect(host, port).await?;
            Ok(s)
        };
        Box::pin(f)
    }
}


impl hyper::client::connect::Connection for AsyncStream {
    fn connected(&self) -> hyper::client::connect::Connected {
        hyper::client::connect::Connected::new()
    }
}

trait AddrForUri {
    fn get_addr<'a>(&'a self) -> anyhow::Result<(&'a str, u16)>;
}
impl AddrForUri for hyper::Uri {
    fn get_addr<'a>(&'a self) -> anyhow::Result<(&'a str, u16)> {
        let host = self.host().ok_or(anyhow::anyhow!("req format error: {:?}", self))?;
        let port = *self.port_u16().get_or_insert_with(|| {
            if Some("https") == self.scheme_str() { 443 } else { 80 }
        });
        Ok((host, port))
    }
}

impl hyper::server::accept::Accept for Listener {
    type Conn = AsyncStream;
    type Error = anyhow::Error;

    fn poll_accept(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<Self::Conn, Self::Error>>> {
        match self.poll_accept_(cx) {
            std::task::Poll::Pending => std::task::Poll::Pending,
            std::task::Poll::Ready(Err(e)) => std::task::Poll::Ready(Some(Err(e))),
            std::task::Poll::Ready(Ok((s, _))) => std::task::Poll::Ready(Some(Ok(s))),
        }
    }
}



