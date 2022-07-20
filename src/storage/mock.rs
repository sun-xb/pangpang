

use super::Storage;
use super::Profile;

use async_trait::async_trait;
use anyhow::{Result, Ok, anyhow};

#[derive(Default, Clone)]
pub struct MockStorage(std::collections::HashMap<String, Profile>);

#[async_trait]
impl<'a> Storage<'a> for MockStorage {
    type S = MockStream<'a>;

    async fn get(&'a self, id: &String) -> Result<&'a Profile> {
        self.0.get(id).ok_or(anyhow!("profile not found"))
    }
    
    async fn put(&mut self, i: Profile) -> Result<()> {
        self.0.insert(i.id(), i);
        Ok(())
    }

    async fn del(&mut self, id: &String) -> Result<Profile> {
        self.0.remove(id).ok_or(anyhow!("profie not found"))
    }

    async fn iter(&'a self) -> Result<Self::S> {
        let i = self.0.iter();
        Ok(MockStream(i))
    }
}

pub struct MockStream<'a>(std::collections::hash_map::Iter<'a, String, Profile>);

impl<'a> tokio_stream::Stream for MockStream<'a> {
    type Item = &'a Profile;

    fn poll_next(self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        std::task::Poll::Ready(self.get_mut().0.next().map(|v| v.1))
    }
}


#[cfg(test)]
mod tests {
    use tokio_stream::StreamExt;

    use super::MockStorage;
    use super::super::Storage;

    #[tokio::test]
    async fn mock() {
        let ms = MockStorage::default();
        let mut s = ms.iter().await.unwrap();
        while let Some(_p) = s.next().await {
        }
    }
}
