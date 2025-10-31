use crate::{apidoc, error::Error, state::SharedAppState};
use axum::{extract::Path, extract::State, http::StatusCode, Json};
use itil_back_db::entities::configuration::changes::{
    self, CIChange, CIChangeCreateset, CIChangeUpdateset,
};
use tracing::info;
use uuid::Uuid;

#[axum::debug_handler]
#[utoipa::path(post,
    path = "/{id}/changes",
    request_body(
        content = CIChangeCreateset,
        description = "Change info necessary for linking.",
        content_type = "application/json",
    ),
    responses(
        (status = CREATED,
            body = CIChange,
            description = "Change created successfully.",
            content_type = "application/json"
        ),
        (status = NOT_FOUND,
            description = "Resource doesn't exist."
        ),
        (status = UNPROCESSABLE_ENTITY,
            description = "Request body didn't pass validations."
        ),
        (status = INTERNAL_SERVER_ERROR,
            description = "Database error."
        )
    ),
    tag = apidoc::CONFIG_ITEMS_TAG
)]
pub async fn create_ci_change(
    State(app_state): State<SharedAppState>,
    Path(ci_id): Path<Uuid>,
    Json(createset): Json<CIChangeCreateset>,
) -> Result<(StatusCode, Json<CIChange>), Error> {
    let relation = changes::create(ci_id, createset, &app_state.db_pool).await?;
    Ok((StatusCode::CREATED, Json(relation)))
}

#[axum::debug_handler]
#[utoipa::path(get,
    path = "/{id}/changes",
    responses(
        (status = OK,
            body = Vec<CIChange>,
            description = "List of changes."
        ),
        (status = NOT_FOUND,
            description = "Resource doesn't exist."
        ),
        (status = INTERNAL_SERVER_ERROR,
            description = "Database error."
        )
    ),
    tag = apidoc::CONFIG_ITEMS_TAG
)]
pub async fn read_all_ci_changes(
    State(app_state): State<SharedAppState>,
    Path(ci_id): Path<Uuid>,
) -> Result<Json<Vec<CIChange>>, Error> {
    let changes = changes::load_all(ci_id, &app_state.db_pool).await?;

    info!("responding with {:?}", changes);

    Ok(Json(changes))
}

#[axum::debug_handler]
#[utoipa::path(get,
    path = "/{id}/changes/{change_id}",
    responses(
        (status = OK,
            body = CIChange,
            description = "OK"
        ),
        (status = NOT_FOUND,
            description = "Record not found in database."
        ),
        (status = INTERNAL_SERVER_ERROR,
            description = "Database error."
        )
    ),
    tag = apidoc::CONFIG_ITEMS_TAG
)]
pub async fn read_one_ci_change(
    State(app_state): State<SharedAppState>,
    Path((ci_id, change_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<CIChange>, Error> {
    let change = changes::load(change_id, ci_id, &app_state.db_pool).await?;
    Ok(Json(change))
}

#[axum::debug_handler]
#[utoipa::path(put,
    path = "/{id}/changes/{change_id}",
    request_body(
        content = CIChangeUpdateset,
        description = "Change data to update in the database.",
        content_type = "application/json",
    ),
    responses(
        (status = OK,
            body = CIChange,
            description = "Change updated successfully.",
            content_type = "application/json"
        ),
        (status = NOT_FOUND,
            description = "Record not found in database."
        ),
        (status = INTERNAL_SERVER_ERROR,
            description = "Database error."
        )
    ),
    tag = apidoc::CONFIG_ITEMS_TAG
)]
pub async fn update_ci_change(
    State(app_state): State<SharedAppState>,
    Path((ci_id, change_id)): Path<(Uuid, Uuid)>,
    Json(updateset): Json<CIChangeUpdateset>,
) -> Result<Json<CIChange>, Error> {
    let change = changes::update(change_id, ci_id, updateset, &app_state.db_pool).await?;
    Ok(Json(change))
}

#[axum::debug_handler]
#[utoipa::path(delete,
    path = "/{id}/changes/{change_id}",
    responses(
        (status = NO_CONTENT,
            description = "Change deleted successfully.",
        ),
        (status = NOT_FOUND,
            description = "Record not found in database."
        ),
        (status = INTERNAL_SERVER_ERROR,
            description = "Database error."
        )
    ),
    tag = apidoc::CONFIG_ITEMS_TAG
)]
pub async fn delete_ci_change(
    State(app_state): State<SharedAppState>,
    Path((ci_id, change_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, Error> {
    changes::delete(change_id, ci_id, &app_state.db_pool).await?;
    Ok(StatusCode::NO_CONTENT)
}
