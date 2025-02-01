use axum::{extract::State, Json};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::instrument;

use crate::error::ApiError;
use crate::types::Id;

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceLookupRequest {
    pub name: String,
    pub barcode: String,
    pub price: f32,
    pub store_id: Id,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecentPrice {
    pub price: f32,
    pub date: Option<NaiveDate>,
}

#[instrument(skip(db))]
pub async fn find_or_create_product(
    db: &PgPool,
    name: &str,
    barcode: &str,
) -> Result<Id, ApiError> {
    let existing = sqlx::query!("SELECT id FROM products WHERE barcode = $1", barcode)
        .fetch_optional(db)
        .await?;

    match existing {
        Some(record) => Ok(record.id),
        None => {
            let record = sqlx::query!(
                "INSERT INTO products (name, barcode) VALUES ($1, $2) RETURNING id",
                name,
                barcode
            )
            .fetch_one(db)
            .await?;
            Ok(record.id)
        }
    }
}

pub async fn insert_price(
    db: &PgPool,
    product_id: Id,
    store_id: Id,
    price: f32,
) -> Result<(), ApiError> {
    sqlx::query!(
        "INSERT INTO prices (product_id, store_id, price, date) 
        VALUES ($1, $2, $3, CURRENT_DATE)
        ON CONFLICT ON CONSTRAINT unique_price_per_day DO NOTHING",
        product_id,
        store_id,
        price
    )
    .execute(db)
    .await?;
    Ok(())
}

#[instrument(skip(db))]
pub async fn get_recent_prices(
    db: &PgPool,
    product_id: Id,
    store_id: Id,
) -> Result<Vec<RecentPrice>, ApiError> {
    let prices = sqlx::query_as!(
        RecentPrice,
        "SELECT price, date FROM prices 
        WHERE product_id = $1 AND store_id = $2 
        ORDER BY date DESC 
        LIMIT 10",
        product_id,
        store_id
    )
    .fetch_all(db)
    .await?;
    Ok(prices)
}

#[instrument(skip(db))]
pub async fn lookup_price(
    State(db): State<PgPool>,
    Json(request): Json<PriceLookupRequest>,
) -> Result<Json<Vec<RecentPrice>>, ApiError> {
    let product_id = find_or_create_product(&db, &request.name, &request.barcode).await?;
    insert_price(&db, product_id, request.store_id, request.price).await?;
    let prices = get_recent_prices(&db, product_id, request.store_id).await?;
    Ok(Json(prices))
}
