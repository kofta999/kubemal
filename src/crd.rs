use k8s_openapi::serde::{Deserialize, Serialize};
use kube::CustomResource;
use schemars::JsonSchema;

#[derive(CustomResource, Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[kube(
    group = "kubemal.kofta.app",
    kind = "Anime",
    version = "v1",
    plural = "animes",
    namespaced
)]
#[serde(rename_all = "camelCase")]
pub struct AnimeSpec {
    pub english_title: Option<String>,
    pub japanese_title: Option<String>,
    pub total_episodes: Option<i32>,
    pub airing_status: Option<AiringStatus>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub enum AiringStatus {
    NotYetAired,
    Airing,
    Finished,
}

#[derive(CustomResource, Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[kube(
    group = "kubemal.kofta.app",
    kind = "WatchRecord",
    version = "v1",
    plural = "watchrecords",
    namespaced,
    status = "WatchRecordStatus"
)]
#[serde(rename_all = "camelCase")]
pub struct WatchRecordSpec {
    pub username: String,
    pub anime_ref: AnimeRef,
    pub episodes_watched: i32,
    pub score: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct AnimeRef {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WatchRecordStatus {
    pub watch_state: WatchState,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub enum WatchState {
    Watching,
    Completed,
    Dropped,
}
