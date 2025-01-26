use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use tracing::instrument;

use crate::error::ApiError;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Store {
    name: String,
    street_number: i32,
    street_name: String,
    city: String,
    country_code: String,
    latitude: f64,
    longitude: f64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct StoreRecord {
    id: i64,
    name: String,
    street_number: i32,
    street_name: String,
    city: String,
    country_code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StoreResponse {
    id: i64,
}

#[instrument(skip(db))]
pub async fn add_store(
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

#[instrument(skip(db))]
pub async fn list_stores(State(db): State<PgPool>) -> Result<Json<Vec<StoreRecord>>, ApiError> {
    let stores = sqlx::query_as!(
        StoreRecord,
        r#"
        SELECT id, name, street_number, street_name, city, country_code
        FROM stores
        ORDER BY name ASC
        "#,
    )
    .fetch_all(&db)
    .await?;
    Ok(Json(stores))
}
