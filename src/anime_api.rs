use crate::crd::{AiringStatus, AnimeSpec};

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
