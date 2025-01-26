use axum::{extract::State, http::StatusCode, Json};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use tracing::instrument;

use crate::error::ApiError;
// Product data structure matching iOS app
#[derive(Debug, Serialize, Deserialize)]
pub struct Product {
    name: String,
    price: f32,
    barcode: String,
}

// Database record with timestamp
#[derive(Serialize, Deserialize, FromRow)]
pub struct ProductRecord {
    id: i64,
    name: String,
    price: f32,
    barcode: String,
    scanned_at: Option<NaiveDate>,
}

#[instrument(skip(db))]
async fn insert_product(db: &PgPool, product: &Product) -> Result<(), ApiError> {
    sqlx::query!(
        r#"INSERT INTO products (name, price, barcode) VALUES ($1, $2, $3)"#,
        product.name,
        product.price,
        product.barcode
    )
    .execute(db)
    .await?;
    Ok(())
}

#[instrument(skip(db))]
async fn fetch_all_products(db: &PgPool) -> Result<Vec<ProductRecord>, ApiError> {
    sqlx::query_as!(
        ProductRecord,
        r#"
         SELECT id, name, price, barcode, scanned_at
         FROM products
         ORDER BY scanned_at DESC
         "#,
    )
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

#[instrument(skip(db))]
pub async fn add_product(
    State(db): State<PgPool>,
    Json(product): Json<Product>,
) -> Result<StatusCode, ApiError> {
    insert_product(&db, &product).await?;
    Ok(StatusCode::CREATED)
}

#[instrument(skip(db))]
pub async fn list_products(State(db): State<PgPool>) -> Result<Json<Vec<ProductRecord>>, ApiError> {
    let products = fetch_all_products(&db).await?;
    Ok(Json(products))
}
