use pangpang::storage::Storage;



#[tokio::main]
async fn main() {
    let profile = pangpang::profile::Profile {
        transport: None,
        protocol: pangpang::profile::Protocol::Ssh(
            pangpang::ssh::Profile {
                host: "127.0.0.1".to_owned(),
                port: 22,
                username: "root".to_owned(),
                password: "123456".to_owned()
            }
        )
    };
    let profile_id = profile.id();
    let mut storage = pangpang::storage::mock::MockStorage::default();
    storage.put(profile).await.unwrap();
    let mgr = pangpang::manager::Manager::new(storage);
    let mut service = mgr.open_http_tunnel("0.0.0.0:1080".parse().unwrap(), None, Some(profile_id)).await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_secs(1000)).await;
    service.shutdown();
}