use kube::{Api, Client};

use crate::crd::{Anime, WatchRecord};

pub async fn get_anime(client: &Client, obj: &WatchRecord) -> Option<Anime> {
    Api::<Anime>::namespaced(
        client.clone(),
        &obj.metadata
            .namespace
            .clone()
            .unwrap_or("default".to_string()),
    )
    .get(&obj.spec.anime_ref.name)
    .await
    .ok()
}
