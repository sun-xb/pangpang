
use std::pin::Pin;

pub enum AsyncStream {
    Tcp(tokio::net::TcpStream),
    Ssh(super::ssh::SSHStream)
}
impl tokio::io::AsyncRead for AsyncStream {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            Self::Tcp(s) => Pin::new(s).poll_read(cx, buf),
            Self::Ssh(s) => Pin::new(s).poll_read(cx, buf)
        }
    }
}

impl tokio::io::AsyncWrite for AsyncStream {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        match self.get_mut() {
            Self::Tcp(s) => Pin::new(s).poll_write(cx, buf),
            Self::Ssh(s) => Pin::new(s).poll_write(cx, buf)
        }
    }

    fn poll_flush(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), std::io::Error>> {
        match self.get_mut() {
            Self::Tcp(s) => Pin::new(s).poll_flush(cx),
            Self::Ssh(s) => Pin::new(s).poll_flush(cx)
        }
    }

    fn poll_shutdown(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), std::io::Error>> {
        match self.get_mut() {
            Self::Tcp(s) => Pin::new(s).poll_shutdown(cx),
            Self::Ssh(s) => Pin::new(s).poll_shutdown(cx)
        }
    }
}

pub enum Transport {
    Direct,
    JumpHost(super::Session)
}

impl Transport {
    pub async fn connect(&mut self, host: &str, port: u16) -> anyhow::Result<AsyncStream> {
        match self {
            Self::Direct => {
                let tcp = tokio::net::TcpStream::connect((host, port)).await?;
                Ok(AsyncStream::Tcp(tcp))
            }
            Self::JumpHost(s) => {
                s.local_tunnel(host, port).await
            }
        }
    }
}