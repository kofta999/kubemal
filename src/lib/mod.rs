use std::{net::SocketAddr, path::PathBuf};

use crate::errors::AppError;
use axum_server::tls_rustls::RustlsConfig;
use kube::Client;
use tracing::info;

mod anime_api;
mod controller;
mod crd;
pub mod errors;
mod router;
mod util;

const PORT: u16 = 3000;

pub async fn run() -> Result<(), AppError> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let config = RustlsConfig::from_pem_file(
        PathBuf::from("secrets/tls.crt"),
        PathBuf::from("secrets/tls.key"),
    )
    .await?;

    let client = Client::try_default().await?;
    info!("Kubernetes client initialized");

    controller::create_controller(client.clone());
    info!("Controller task started");

    info!(port = PORT, "Starting webhook server");

    let app = router::create_router(client).await;
    let addr = SocketAddr::from(([127, 0, 0, 1], PORT));

    axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
