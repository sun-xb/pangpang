

use russh::client;
use russh_keys::key;

pub struct Handler;

impl client::Handler for Handler {
    type Error = russh::Error;

    type FutureBool = std::future::Ready<Result<(Self, bool), Self::Error>>;

    type FutureUnit = std::future::Ready<Result<(Self, russh::client::Session), Self::Error>>;

    fn finished_bool(self, b: bool) -> Self::FutureBool {
        std::future::ready(Ok((self, b)))
    }

    fn finished(self, session: client::Session) -> Self::FutureUnit {
        std::future::ready(Ok((self, session)))
    }

    fn check_server_key(self, server_public_key: &key::PublicKey) -> Self::FutureBool {
        log::debug!("server public key {:?}", server_public_key);
        self.finished_bool(true)
    }

    fn server_channel_open_forwarded_tcpip(
        self,
        channel: russh::Channel<russh::client::Msg>,
        connected_address: &str,
        connected_port: u32,
        originator_address: &str,
        originator_port: u32,
        session: russh::client::Session,
    ) -> Self::FutureUnit {
        log::debug!("new channel: {} {} {} {}", connected_address, connected_port, originator_address, originator_port);
        self.finished(session)
    }
}