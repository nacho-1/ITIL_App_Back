use crate::{apidoc, error::Error, state::SharedAppState};
use axum::{extract::Path, extract::State, http::StatusCode, Json};
use itil_back_db::entities::changes::{self, RFCCreateset, RFCUpdateset, RFC};
use tracing::info;
use uuid::Uuid;

pub mod incident_relations;

#[axum::debug_handler]
#[utoipa::path(post,
    path = "",
    request_body(
        content = RFCCreateset,
        description = "RFC to create in the database.",
        content_type = "application/json",
    ),
    responses(
        (status = CREATED,
            body = RFC,
            description = "RFC created successfully.",
            content_type = "application/json"
        ),
        (status = UNPROCESSABLE_ENTITY,
            description = "Request body didn't pass validations."
        ),
        (status = INTERNAL_SERVER_ERROR,
            description = "Database error."
        )
    ),
    tag = apidoc::CHANGES_TAG
)]
pub async fn create_rfc(
    State(app_state): State<SharedAppState>,
    Json(createset): Json<RFCCreateset>,
) -> Result<(StatusCode, Json<RFC>), Error> {
    let rfc = changes::create(createset, &app_state.db_pool).await?;
    Ok((StatusCode::CREATED, Json(rfc)))
}

#[axum::debug_handler]
#[utoipa::path(get,
    path = "",
    responses(
        (status = OK,
            body = Vec<RFC>,
            description = "List of RFCs."
        ),
        (status = INTERNAL_SERVER_ERROR,
            description = "Database error."
        )
    ),
    tag = apidoc::CHANGES_TAG
)]
pub async fn read_all_rfcs(
    State(app_state): State<SharedAppState>,
) -> Result<Json<Vec<RFC>>, Error> {
    let rfcs = changes::load_all(&app_state.db_pool).await?;

    info!("responding with {:?}", rfcs);

    Ok(Json(rfcs))
}

#[axum::debug_handler]
#[utoipa::path(get,
    path = "/{id}",
    responses(
        (status = OK,
            body = RFC,
            description = "OK"
        ),
        (status = NOT_FOUND,
            description = "Record not found in database."
        ),
        (status = INTERNAL_SERVER_ERROR,
            description = "Database error."
        )
    ),
    tag = apidoc::CHANGES_TAG
)]
pub async fn read_one_rfc(
    State(app_state): State<SharedAppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<RFC>, Error> {
    let rfc = changes::load(id, &app_state.db_pool).await?;
    Ok(Json(rfc))
}

#[axum::debug_handler]
#[utoipa::path(put,
    path = "/{id}",
    request_body(
        content = RFCUpdateset,
        description = "RFC data to update in the database.",
        content_type = "application/json",
    ),
    responses(
        (status = OK,
            body = RFC,
            description = "RFC updated successfully.",
            content_type = "application/json"
        ),
        (status = UNPROCESSABLE_ENTITY,
            description = "Request body didn't pass validations."
        ),
        (status = NOT_FOUND,
            description = "Record not found in database."
        ),
        (status = INTERNAL_SERVER_ERROR,
            description = "Database error."
        )
    ),
    tag = apidoc::CHANGES_TAG
)]
pub async fn update_rfc(
    State(app_state): State<SharedAppState>,
    Path(id): Path<Uuid>,
    Json(updateset): Json<RFCUpdateset>,
) -> Result<Json<RFC>, Error> {
    let rfc = changes::update(id, updateset, &app_state.db_pool).await?;
    Ok(Json(rfc))
}

#[axum::debug_handler]
#[utoipa::path(delete,
    path = "/{id}",
    responses(
        (status = NO_CONTENT,
            description = "RFC deleted successfully.",
        ),
        (status = NOT_FOUND,
            description = "Record not found in database."
        ),
        (status = INTERNAL_SERVER_ERROR,
            description = "Database error."
        )
    ),
    tag = apidoc::CHANGES_TAG
)]
pub async fn delete_rfc(
    State(app_state): State<SharedAppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, Error> {
    changes::delete(id, &app_state.db_pool).await?;
    Ok(StatusCode::NO_CONTENT)
}
