use std::future::Future;

use russh::ChannelMsg;
use tokio::io::{AsyncWrite, AsyncRead};



pub struct SSHStream {
    pub(super) channel: russh::Channel<russh::client::Msg>,
    read_buf: Vec<u8>,
    buffer_offset: usize,
    buffer_end: usize,
}

impl From<russh::Channel<russh::client::Msg>> for SSHStream {
    fn from(ch: russh::Channel<russh::client::Msg>) -> Self {
        Self {
            channel: ch,
            read_buf: Vec::new(),
            buffer_offset: 0,
            buffer_end: 0,
        }
    }
}

impl AsyncWrite for SSHStream {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        match Box::pin(self.get_mut().channel.data(buf)).as_mut().poll(cx) {
            std::task::Poll::Ready(Ok(_)) => std::task::Poll::Ready(Ok(buf.len())),
            std::task::Poll::Ready(Err(e)) => {
                std::task::Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, e.to_string())))
            }
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        match Box::pin(self.get_mut().channel.eof()).as_mut().poll(cx) {
            std::task::Poll::Ready(Ok(_)) => std::task::Poll::Ready(Ok(())),
            std::task::Poll::Ready(Err(e)) => {
                std::task::Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, e.to_string())))
            }
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

impl AsyncRead for SSHStream {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let mut this = self.get_mut();
        loop {
            let remaining = this.buffer_end - this.buffer_offset;
            if remaining > 0 {
                let length = remaining.min(buf.remaining());
                let offset = this.buffer_offset;
                this.buffer_offset += length;
                buf.put_slice(&this.read_buf[offset..this.buffer_offset]);
                return std::task::Poll::Ready(Ok(()));
            }
            return match Box::pin(this.channel.wait()).as_mut().poll(cx) {
                std::task::Poll::Ready(Some(msg)) => match msg {
                    ChannelMsg::Data { data } => {
                        let buf = data.as_ref();
                        this.buffer_offset = 0;
                        this.buffer_end = buf.len();
                        if buf.len() > this.read_buf.len() {
                            this.read_buf = data.to_vec();
                        } else {
                            this.read_buf[..buf.len()].copy_from_slice(buf);
                        }
                        continue
                    }
                    ChannelMsg::Eof | ChannelMsg::Close => {
                        let e = std::io::Error::from(std::io::ErrorKind::UnexpectedEof);
                        std::task::Poll::Ready(Err(e))
                    }
                    _ => continue,
                },
                std::task::Poll::Ready(None) => std::task::Poll::Ready(Ok(())),
                std::task::Poll::Pending => std::task::Poll::Pending,
            };
        }
    }
}
