

use super::profile::Profile;

use async_trait::async_trait;
use tokio_stream::Stream;
use anyhow::Result;


pub mod mock;

#[async_trait]
pub trait Storage<'a>: Send + Sync + Clone {
    type S: Stream<Item = &'a Profile> + 'a;

    async fn get(&'a self, id: &String) -> Result<&'a Profile>;
    async fn put(&mut self, i: Profile) -> Result<()>;
    async fn del(&mut self, id: &String) -> Result<Profile>;
    async fn iter(&'a self) -> Result<Self::S>;
}


