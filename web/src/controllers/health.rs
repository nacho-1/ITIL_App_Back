use axum::http::StatusCode;

/// Get health of the API.
#[utoipa::path(
    method(get, head),
    path = "/api/health",
    responses(
        (status = OK, description = "Success")
    )
)]
pub async fn health() -> StatusCode {
    StatusCode::OK
}
