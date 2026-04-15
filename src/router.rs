use crate::crd::{AiringStatus, Anime, AnimeSpec};
use axum::{Json, Router, extract, routing::post};
use json_patch::Patch;
use kube::{
    api::DynamicObject,
    core::admission::{AdmissionRequest, AdmissionResponse, AdmissionReview, Operation},
};

const ANILIST_URL: &str = "https://graphql.anilist.co";
const ANILIST_MEDIA_QUERY: &str = r#"
query ($search: String) {
  Media(search: $search, type: ANIME) {
    title {
      english
      romaji
      native
    }
    episodes
    status
  }
}
"#;

pub fn create_router() -> Router {
    Router::new().route("/mutate", post(mutation_handler))
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
        && let Some(new_spec) = fetch_anime_details(&obj.metadata.name.unwrap()).await
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

pub async fn fetch_anime_details(english_title: &str) -> Option<AnimeSpec> {
    let client = reqwest::Client::new();
    let json = client
        .post(ANILIST_URL)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&serde_json::json!({
            "query": ANILIST_MEDIA_QUERY,
            "variables": { "search": english_title }
        }))
        .send()
        .await
        .ok()?
        .json::<serde_json::Value>()
        .await
        .ok()?;

    let media = &json["data"]["Media"];
    let english = media["title"]["english"].as_str();
    let romaji = media["title"]["romaji"].as_str();
    let native = media["title"]["native"].as_str();

    Some(AnimeSpec {
        english_title: Some(english.or(romaji).unwrap_or(english_title).to_string()),
        japanese_title: native.map(|s| s.to_string()),
        total_episodes: media["episodes"].as_i64().map(|x| x as i32),
        airing_status: match media["status"].as_str() {
            Some("RELEASING") => Some(AiringStatus::Airing),
            Some("FINISHED") => Some(AiringStatus::Finished),
            Some("NOT_YET_RELEASED") => Some(AiringStatus::NotYetAired),
            _ => None,
        },
    })
}
