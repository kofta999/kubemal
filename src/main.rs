use kube::Client;
use kubemal::errors::AppError;

const PORT: u16 = 3000;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{PORT}")).await?;
    let client = Client::try_default().await?;

    kubemal::controller::create_controller(client.clone());

    axum::serve(listener, kubemal::router::create_router(client).await).await?;

    Ok(())
}
