pub mod service;
mod watch;

use std::{convert::TryInto, marker::PhantomData};

use super::{core::registry::Registry, handler, impl_request_handler};
use async_trait::async_trait;
use common::errors::SvcError;
use mbus_api::{v0::*, *};

pub(crate) fn configure(builder: common::Service) -> common::Service {
    let registry = builder.get_shared_state::<Registry>().clone();
    builder
        .with_channel(ChannelVs::Watcher)
        .with_default_liveness()
        .with_shared_state(service::Service::new(registry))
        .with_subscription(handler!(CreateWatch))
        .with_subscription(handler!(GetWatchers))
        .with_subscription(handler!(DeleteWatch))
}

#[cfg(test)]
mod tests {
    use once_cell::sync::OnceCell;
    use std::{net::SocketAddr, str::FromStr, time::Duration};
    use store::{etcd::Etcd, store::Store, types::v0::volume::VolumeState};
    use testlib::*;
    use tokio::net::TcpStream;

    static CALLBACK: OnceCell<tokio::sync::mpsc::Sender<()>> = OnceCell::new();

    async fn setup_watcher() -> (
        store::types::v0::volume::VolumeState,
        tokio::sync::mpsc::Receiver<()>,
    ) {
        let volume = VolumeState {
            uuid: "9c00e112-6548-4813-a7bd-6881f27841ee".into(),
            size: 20 * 1024 * 1024,
            labels: vec![],
            num_replicas: 2,
            protocol: Default::default(),
            nexuses: vec![],
            num_paths: 0,
            state: Default::default(),
        };

        let (s, r) = tokio::sync::mpsc::channel(1);
        CALLBACK.set(s).unwrap();

        async fn notify() -> actix_web::HttpResponse {
            CALLBACK.get().cloned().unwrap().send(()).await.unwrap();
            actix_web::HttpResponse::Ok().finish()
        }

        actix_rt::spawn(async move {
            let _ = actix_web::HttpServer::new(|| {
                actix_web::App::new().service(
                    actix_web::web::resource("/test").route(actix_web::web::put().to(notify)),
                )
            })
            .bind("10.1.0.1:8082")
            .unwrap()
            .workers(1)
            .run()
            .await;
        });

        // wait until the "callback" server is running
        callback_server_liveness("10.1.0.1:8082").await;

        (volume, r)
    }

    async fn callback_server_liveness(uri: &str) {
        let sa = SocketAddr::from_str(uri).unwrap();
        for _ in 0 .. 25 {
            if TcpStream::connect(&sa).await.is_ok() {
                return;
            }
            tokio::time::delay_for(std::time::Duration::from_millis(10)).await;
        }
        TcpStream::connect(&sa).await.unwrap();
    }

    #[actix_rt::test]
    async fn watcher() {
        let cluster = ClusterBuilder::builder().with_pools(1).build().await;
        let cluster = cluster.unwrap();
        let client = cluster.rest_v0();

        let (mut volume, mut callback_ch) = setup_watcher().await;
        let volume_watch_id = v0::WatchResourceId::VolumeState(volume.uuid.clone());
        let callback = url::Url::parse("http://10.1.0.1:8082/test").unwrap();

        let watchers = match client.get_watches(volume_watch_id.clone()).await {
            Ok(w) => w,
            Err(e) => panic!("Error: {:?}", e),
        };
        assert!(watchers.is_empty());

        let mut store = Etcd::new("0.0.0.0:2379")
            .await
            .expect("Failed to connect to etcd.");

        client
            .create_watch(volume_watch_id.clone(), callback.clone())
            .await
            .expect_err("volume does not exist in the store");

        store.put_obj(&volume).await.unwrap();

        client
            .create_watch(volume_watch_id.clone(), callback.clone())
            .await
            .unwrap();

        let watchers = client.get_watches(volume_watch_id.clone()).await.unwrap();
        assert_eq!(
            watchers.first(),
            Some(&v0::RestWatch {
                resource: volume_watch_id.to_string(),
                callback: callback.to_string(),
            })
        );
        assert_eq!(watchers.len(), 1);

        // Put the volume in the store again and check that the watch callback
        // is called.
        store.put_obj(&volume).await.unwrap();
        tokio::time::timeout(Duration::from_millis(250), callback_ch.recv())
            .await
            .unwrap();

        // Stop watching changes to the volume.
        client
            .delete_watch(volume_watch_id.clone(), callback.clone())
            .await
            .unwrap();

        // Put updated volume to the store and check that the callback is not
        // called.
        volume.num_paths = 2;
        store.put_obj(&volume).await.unwrap();
        tokio::time::timeout(Duration::from_millis(250), callback_ch.recv())
            .await
            .expect_err("should have been deleted so no callback");

        let watchers = client.get_watches(volume_watch_id).await.unwrap();
        assert!(watchers.is_empty());
    }
}
