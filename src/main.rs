use crate::crd::{Anime, WatchRecord};
use kube::{Api, Client, ResourceExt, api::ListParams};

mod crd;

#[tokio::main]
async fn main() {
    let client = Client::try_default().await.unwrap();
    let animes: Api<Anime> = Api::default_namespaced(client.clone());
    let watch_records: Api<WatchRecord> = Api::default_namespaced(client);

    for p in animes.list(&ListParams::default()).await.unwrap() {
        dbg!(p.name_any());
    }

    for p in watch_records.list(&ListParams::default()).await.unwrap() {
        dbg!(p.name_any());
    }
}
