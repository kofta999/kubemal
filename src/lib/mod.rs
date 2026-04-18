use std::{net::SocketAddr, path::PathBuf};

use crate::errors::AppError;
use axum_server::tls_rustls::RustlsConfig;
use kube::Client;
use tracing::{error, info};

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

    let config =
        RustlsConfig::from_pem_file(PathBuf::from("tls/tls.crt"), PathBuf::from("tls/tls.key"))
            .await?;

    let client = Client::try_default().await?;
    info!("Kubernetes client initialized");

    let mut controller_task = controller::create_controller(client.clone());
    info!("Controller task started");

    info!(port = PORT, "Starting webhook server");

    let app = router::create_router(client).await;
    let addr = SocketAddr::from(([0, 0, 0, 0], PORT));

    let mut webhook_server =
        Box::pin(axum_server::bind_rustls(addr, config).serve(app.into_make_service()));

    tokio::select! {
        server_result = &mut webhook_server => {
            server_result?;
        }
        controller_result = &mut controller_task => {
            match controller_result {
                Ok(()) => {
                    error!("Controller task exited unexpectedly");
                    return Err(AppError::ControllerStopped);
                }
                Err(e) => {
                    error!(error = %e, "Controller task failed");
                    return Err(AppError::ControllerJoin(e));
                }
            }
        }
    }

    Ok(())
}
