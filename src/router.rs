use crate::crd::{AiringStatus, Anime, AnimeSpec};
use axum::{Json, Router, extract, routing::post};
use json_patch::Patch;
use kube::{
    api::DynamicObject,
    core::admission::{AdmissionRequest, AdmissionResponse, AdmissionReview, Operation},
};

const JIKAN_URL: &str = "https://api.jikan.moe/v4";

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
        && let Some(obj) = req.object.as_ref()
        && let Some(new_spec) = fetch_anime_details(&obj.spec.english_title).await
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
    let url = reqwest::Url::parse_with_params(
        format!("{JIKAN_URL}/anime").as_str(),
        &[("q", english_title), ("limit", "1")],
    )
    .ok()?;

    let json = reqwest::get(url)
        .await
        .ok()?
        .json::<serde_json::Value>()
        .await
        .ok()?;

    Some(AnimeSpec {
        english_title: json["data"][0]["title_english"]
            .as_str()
            .unwrap_or(english_title)
            .to_string(),
        japanese_title: json["data"][0]["title_japanese"]
            .as_str()
            .map(|s| s.to_string()),
        total_episodes: json["data"][0]["episodes"].as_i64().map(|x| x as i32),
        airing_status: match json["data"][0]["status"].as_str() {
            Some("Currently Airing") => Some(AiringStatus::Airing),
            Some("Finished Airing") => Some(AiringStatus::Finished),
            Some("Not yet aired") => Some(AiringStatus::NotYetAired),
            _ => None,
        },
    })
}
