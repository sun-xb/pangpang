
use anyhow::Result;


pub enum Listener {
    Ssh(super::ssh::RemoteListener),
    Local(tokio::net::TcpListener)
}


impl Listener {
    pub async fn accept(&mut self) -> Result<(super::AsyncStream, std::net::SocketAddr)> {
        match self {
            Self::Ssh(l) => {
                let (stream, addr) = l.accept().await?;
                Ok((super::AsyncStream::Ssh(stream), addr))
            }
            Self::Local(l) => {
                let (stream, addr) = l.accept().await?;
                Ok((super::AsyncStream::Tcp(stream), addr))
            }
        }
    }

    pub fn poll_accept_(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>)
        -> std::task::Poll<Result<(super::AsyncStream, std::net::SocketAddr)>> {
        match self.get_mut() {
            Self::Ssh(l) => {
                match l.poll_accept(cx) {
                    std::task::Poll::Pending => std::task::Poll::Pending,
                    std::task::Poll::Ready(Err(e)) => std::task::Poll::Ready(Err(anyhow::Error::from(e))),
                    std::task::Poll::Ready(Ok((s, a))) => std::task::Poll::Ready(Ok((super::AsyncStream::Ssh(s), a)))
                }
            }
            Self::Local(l) => {
                match l.poll_accept(cx) {
                    std::task::Poll::Pending => std::task::Poll::Pending,
                    std::task::Poll::Ready(Err(e)) => std::task::Poll::Ready(Err(anyhow::Error::from(e))),
                    std::task::Poll::Ready(Ok((s, a))) => std::task::Poll::Ready(Ok((super::AsyncStream::Tcp(s), a)))
                }
            }
        }
    }

    pub async fn close(&mut self) -> Result<()> {
        match self {
            Self::Ssh(l) => l.close().await?,
            Self::Local(_l) => (),
        };
        Ok(())
    }

}

