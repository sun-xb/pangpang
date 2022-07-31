

use tokio::io::{AsyncRead, AsyncWrite};
use super::SSHStream;

pub struct SSHPty(SSHStream);

impl SSHPty {
    pub async fn new(mut ch: russh::Channel<russh::client::Msg>) -> anyhow::Result<Self> {
        ch.request_pty(false, "xterm-color256", 80, 30, 1, 1, &[]).await?;
        ch.request_shell(false).await?;
        Ok(Self(SSHStream::from(ch)))
    }

    pub async fn resize(&mut self, width: i16, height: i16) -> anyhow::Result<()> {
        self.0.channel.window_change(width as u32, height as u32, 1, 1).await?;
        Ok(())
    }
}

impl AsyncRead for SSHPty {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.get_mut().0).poll_read(cx, buf)
    }
}

impl AsyncWrite for SSHPty {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        std::pin::Pin::new(&mut self.get_mut().0).poll_write(cx, buf)
    }

    fn poll_flush(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), std::io::Error>> {
        std::pin::Pin::new(&mut self.get_mut().0).poll_flush(cx)
    }

    fn poll_shutdown(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), std::io::Error>> {
        std::pin::Pin::new(&mut self.get_mut().0).poll_shutdown(cx)
    }
}