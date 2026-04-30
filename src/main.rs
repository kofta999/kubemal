use kubemal::errors::AppError;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("kubemal=info,kube=info,tower_http=info")),
        )
        .with_target(false)
        .compact()
        .init();

    kubemal::run().await?;

    Ok(())
}
