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

    #[serde(rename = "storeID")]
    pub store_id: Id,

    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentPrice {
    pub name: String,
    pub price: f32,
    pub absolute_price_change: Option<f32>,
    pub relative_price_change: Option<f32>,
    pub date: Option<NaiveDate>,
    pub store_name: Option<String>,
    pub distance: Option<f64>,
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
    request: PriceLookupRequest,
) -> Result<Vec<RecentPrice>, ApiError> {
    let prices = sqlx::query_as!(
        RecentPrice,
        "SELECT
            pr.name,
            date,
            p.price,
            (p.price - $5) AS absolute_price_change,
            ((p.price - $5) / p.price) AS relative_price_change,
            s.name as store_name,
            ST_Distance(
                s.coordinate, 
                ST_SetSRID(ST_Point($1, $2), 4326)
            ) as distance
        FROM
            prices AS p
        JOIN
            stores AS s ON p.store_id = s.id
        JOIN
            products AS pr ON p.product_id = pr.id
        WHERE
            pr.barcode = $4
            AND ST_Distance(
                s.coordinate, 
                ST_SetSRID(ST_Point($1, $2), 4326)
            ) <= 1000
            AND s.id != $3
        ORDER BY
            price ASC, date DESC",
        request.latitude,
        request.longitude,
        request.store_id,
        request.barcode,
        request.price,
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
    let prices = get_recent_prices(&db, request).await?;
    Ok(Json(prices))
}
