use crate::crd::{Anime, WatchRecord, WatchState};
use kube::{Api, Client};

pub async fn get_anime(client: &Client, obj: &WatchRecord) -> Option<Anime> {
    Api::<Anime>::namespaced(client.clone(), &obj.metadata.namespace.clone()?)
        .get(&obj.spec.anime_ref.name)
        .await
        .ok()
}

pub fn calc_watch_state(watch_record: &WatchRecord, total_eps: i32) -> WatchState {
    let watched_eps = watch_record.spec.episodes_watched;

    match watch_record.spec.status.as_ref() {
        Some(status) if watched_eps != total_eps => status.clone().into(),
        _ => {
            if watched_eps == total_eps {
                WatchState::Completed
            } else {
                WatchState::Watching
            }
        }
    }
}
