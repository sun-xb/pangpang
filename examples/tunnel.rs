use pangpang::storage::Storage;



#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Trace).unwrap();

    let profile = pangpang::profile::Profile {
        transport: None,
        protocol: pangpang::profile::Protocol::Ssh(
            pangpang::ssh::Profile {
                host: "127.0.0.1".to_owned(),
                port: 22,
                username: "sun".to_owned(),
                password: "123456".to_owned()
            }
        )
    };
    let profile_id = profile.id();
    let mut storage = pangpang::storage::mock::MockStorage::default();
    storage.put(profile).await.unwrap();
    let mgr = pangpang::manager::Manager::new(storage);
    let mut sess = mgr.open_session(&profile_id).await.unwrap();
    sess.remote_listener("0.0.0.0:8080".parse().unwrap()).await.unwrap();
    let service = mgr.open_http_tunnel("0.0.0.0:1080".parse().unwrap(), None, Some(profile_id)).await.unwrap();
    tokio::signal::ctrl_c().await.unwrap();
    service.shutdown();
}