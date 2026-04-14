mod crd;
mod router;

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, router::create_router())
        .await
        .unwrap();

    // let client = Client::try_default().await.unwrap();
    // let animes: Api<Anime> = Api::default_namespaced(client.clone());
    // let watch_records: Api<WatchRecord> = Api::default_namespaced(client);

    // for p in animes.list(&ListParams::default()).await.unwrap() {
    //     dbg!(p.name_any());
    // }

    // for p in watch_records.list(&ListParams::default()).await.unwrap() {
    //     dbg!(p.name_any());
    // }
}
