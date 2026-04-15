use crate::{
    anime_api,
    crd::{Anime, WatchRecord},
};
use axum::{Json, Router, extract, routing::post};
use json_patch::Patch;
use kube::{
    Api, Client,
    api::DynamicObject,
    core::admission::{AdmissionRequest, AdmissionResponse, AdmissionReview, Operation},
};

pub async fn create_router() -> Router {
    let client = Client::try_default().await.unwrap();
    let state = client;

    Router::new()
        .route("/mutate", post(mutation_handler))
        .route("/validate", post(validation_handler))
        .with_state(state)
}

async fn validation_handler(
    extract::State(client): extract::State<Client>,
    extract::Json(payload): extract::Json<AdmissionReview<WatchRecord>>,
) -> Json<AdmissionReview<DynamicObject>> {
    let req: AdmissionRequest<WatchRecord> = match payload.try_into() {
        Ok(r) => r,
        Err(e) => {
            let rev = AdmissionResponse::invalid(e.to_string()).into_review();
            return Json(rev);
        }
    };

    let mut resp = AdmissionResponse::from(&req);

    if req.operation == Operation::Create
        && let Some(obj) = req.object
        && let Some(anime) = Api::<Anime>::namespaced(
            client.clone(),
            &obj.metadata.namespace.unwrap_or("default".to_string()),
        )
        .get(&obj.spec.anime_ref.name)
        .await
        .ok()
    {
        let watched_eps = obj.spec.episodes_watched;
        let anime_eps = anime.spec.total_episodes.expect("Must be there");

        if watched_eps <= 0 {
            resp = resp.deny("Total episodes is less or equal than zero")
        }

        if watched_eps > anime_eps {
            resp = resp.deny(format!(
                "Total episodes {} is more than anime episodes {}",
                watched_eps, anime_eps
            ))
        }
    }

    let review = resp.into_review();

    Json(review)
}

async fn mutation_handler(
    extract::Json(payload): extract::Json<AdmissionReview<Anime>>,
) -> Json<AdmissionReview<DynamicObject>> {
    let types = payload.types.clone();

    let req: AdmissionRequest<Anime> = match payload.try_into() {
        Ok(r) => r,
        Err(e) => {
            let mut rev = AdmissionResponse::invalid(e.to_string()).into_review();
            rev.types = types;
            return Json(rev);
        }
    };

    let mut resp = AdmissionResponse::from(&req);

    if req.operation == Operation::Create
        && let Some(obj) = req.object
        && let Some(new_spec) = anime_api::fetch_anime_details(&obj.metadata.name.unwrap()).await
    {
        let patch_value = serde_json::json!([
            { "op": "replace", "path": "/spec/englishTitle", "value": new_spec.english_title },
            { "op": "replace", "path": "/spec/japaneseTitle", "value": new_spec.japanese_title },
            { "op": "replace", "path": "/spec/totalEpisodes", "value": new_spec.total_episodes },
            { "op": "replace", "path": "/spec/airingStatus", "value": new_spec.airing_status }
        ]);

        let patch: Patch = serde_json::from_value(patch_value).expect("valid patch");

        resp = resp.with_patch(patch).unwrap();
    }

    let review = resp.into_review();

    Json(review)
}
