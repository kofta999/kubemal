use kube::Client;

use crate::controller::create_controller;

mod anime_api;
mod controller;
mod crd;
mod router;
mod util;

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    let client = Client::try_default().await.unwrap();

    create_controller(client.clone());

    axum::serve(listener, router::create_router(client).await)
        .await
        .unwrap();
}
