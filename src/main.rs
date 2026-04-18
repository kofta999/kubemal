use kube::Client;
use kubemal::errors::AppError;
use tracing::info;
use tracing_subscriber::EnvFilter;

const PORT: u16 = 3000;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("kubemal=info,kube=info")),
        )
        .with_target(false)
        .compact()
        .init();

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{PORT}")).await?;
    info!(port = PORT, "TCP listener bound");

    let client = Client::try_default().await?;
    info!("Kubernetes client initialized");

    kubemal::controller::create_controller(client.clone());
    info!("Controller task started");

    info!(port = PORT, "Starting webhook server");

    axum::serve(listener, kubemal::router::create_router(client).await).await?;

    Ok(())
}
