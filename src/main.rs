use axum::{
    http::StatusCode,
    routing::{get, post},
    Router,
};

use sqlx::PgPool;
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::instrument;
use tracing_subscriber::EnvFilter;

mod error;
mod products;
mod stores;

use crate::products::{add_product, list_products};
use crate::stores::{add_store, list_stores};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .or_else(|_| EnvFilter::try_new("axum_tracing_example=error,tower_http=warn"))
                .unwrap(),
        )
        .init();
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Database connection
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");
    let db = PgPool::connect(&database_url).await?;

    // Create API router
    let app = Router::new()
        .route("/healthcheck", get(healthcheck))
        .route("/products", get(list_products))
        .route("/products", post(add_product))
        .route("/stores", post(add_store))
        .route("/stores", get(list_stores))
        .layer(CorsLayer::permissive()) // Enable CORS for development
        .layer(TraceLayer::new_for_http())
        .with_state(db);

    // Start server
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("Server starting on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

#[instrument]
async fn healthcheck() -> StatusCode {
    StatusCode::OK
}
