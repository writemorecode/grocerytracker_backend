use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub enum ApiError {
    DatabaseError(sqlx::Error),
}
impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        ApiError::DatabaseError(err)
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::DatabaseError(err) => {
                tracing::error!("{}", err.to_string());
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
            }
        }
    }
}
