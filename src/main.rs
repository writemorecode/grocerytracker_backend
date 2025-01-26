use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::instrument;
use tracing_subscriber::EnvFilter;
mod error;
use crate::error::ApiError;

// Product data structure matching iOS app
#[derive(Debug, Serialize, Deserialize)]
struct Product {
    name: String,
    price: f32,
    barcode: String,
}

// Database record with timestamp
#[derive(Serialize, Deserialize, FromRow)]
struct ProductRecord {
    id: i64,
    name: String,
    price: f32,
    barcode: String,
    scanned_at: Option<NaiveDate>,
}

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

// Handler for POST /products
#[instrument(skip(db))]
async fn add_product(
    State(db): State<PgPool>,
    Json(product): Json<Product>,
) -> Result<StatusCode, ApiError> {
    sqlx::query!(
        r#"INSERT INTO products (name, price, barcode) VALUES ($1, $2, $3)"#,
        product.name,
        product.price,
        product.barcode
    )
    .execute(&db)
    .await?;
    Ok(StatusCode::CREATED)
}

// Handler for GET /products
#[instrument(skip(db))]
async fn list_products(State(db): State<PgPool>) -> Result<Json<Vec<ProductRecord>>, ApiError> {
    let products = sqlx::query_as!(
        ProductRecord,
        r#"
         SELECT id, name, price, barcode, scanned_at
         FROM products
         ORDER BY scanned_at DESC
         "#,
    )
    .fetch_all(&db)
    .await?;
    Ok(Json(products))
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
struct Store {
    name: String,
    street_number: i32,
    street_name: String,
    city: String,
    country_code: String,
    latitude: f64,
    longitude: f64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
struct StoreRecord {
    id: i64,
    name: String,
    street_number: i32,
    street_name: String,
    city: String,
    country_code: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct StoreResponse {
    id: i64,
}

#[instrument(skip(db))]
async fn add_store(
    State(db): State<PgPool>,
    Json(store): Json<Store>,
) -> Result<Json<StoreResponse>, ApiError> {
    let store_id = sqlx::query_as!(StoreResponse,
        "SELECT id FROM stores WHERE street_number = $1 AND street_name = $2 AND city = $3 AND country_code = $4",
        store.street_number, store.street_name, store.city, store.country_code)
        .fetch_optional(&db)
        .await?;

    if let Some(store) = store_id {
        return Ok(Json(store));
    } else {
        let store_id = sqlx::query_as!(
            StoreResponse,
            r#"
            INSERT INTO stores (name, street_number, street_name, city, country_code, coordinate)
            VALUES ($1,$2,$3,$4,$5,
                    ST_SetSRID(ST_MakePoint($6, $7), 4326)
            )
            RETURNING id
            "#,
            store.name,
            store.street_number,
            store.street_name,
            store.city,
            store.country_code,
            store.longitude,
            store.latitude,
        )
        .fetch_one(&db)
        .await?;
        Ok(Json(store_id))
    }
}
