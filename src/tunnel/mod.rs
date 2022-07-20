

mod http;
pub use http::Server as Http;

use super::storage::Storage;
use super::manager::Manager;

use super::session::AsyncStream;



#[derive(Clone)]
pub struct StreamConnector<S: for<'a> Storage<'a> + 'static> {
    mgr: Manager<S>,
    id: Option<String>,
}

impl<S: for<'a> Storage<'a> + 'static> StreamConnector<S> {
    pub fn new(mgr: Manager<S>, id: Option<String>) -> Self {
        Self { mgr, id }
    }
    
    pub async fn connect(&self, host: &str, port: u16) -> anyhow::Result<AsyncStream> {
        self.mgr.open_transport(self.id.as_ref()).await?.connect(host, port).await
    }
}