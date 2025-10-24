use crate::{apidoc, error::Error, state::SharedAppState};
use axum::{extract::Path, extract::State, http::StatusCode, Json};
use itil_back_db::entities::configitems::{
    self, ConfigItem, ConfigItemCreateset, ConfigItemUpdateset,
};
use tracing::info;
use uuid::Uuid;

#[axum::debug_handler]
#[utoipa::path(post,
    path = "",
    request_body(
        content = ConfigItemCreateset,
        description = "Configuration Item to create in the database.",
        content_type = "application/json",
    ),
    responses(
        (status = CREATED,
            body = ConfigItem,
            description = "Configuration Item created successfully.",
            content_type = "application/json"
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
pub async fn create_ci(
    State(app_state): State<SharedAppState>,
    Json(configitem): Json<ConfigItemCreateset>,
) -> Result<(StatusCode, Json<ConfigItem>), Error> {
    let configitem = configitems::create(configitem, &app_state.db_pool).await?;
    Ok((StatusCode::CREATED, Json(configitem)))
}

#[axum::debug_handler]
#[utoipa::path(get,
    path = "",
    responses(
        (status = OK,
            body = Vec<ConfigItem>,
            description = "List of Configuration Items."
        ),
        (status = INTERNAL_SERVER_ERROR,
            description = "Database error."
        )
    ),
    tag = apidoc::CONFIG_ITEMS_TAG
)]
pub async fn read_all_ci(
    State(app_state): State<SharedAppState>,
) -> Result<Json<Vec<ConfigItem>>, Error> {
    let configitems = configitems::load_all(&app_state.db_pool).await?;

    info!("responding with {:?}", configitems);

    Ok(Json(configitems))
}

#[axum::debug_handler]
#[utoipa::path(get,
    path = "/{id}",
    responses(
        (status = OK,
            body = ConfigItem,
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
pub async fn read_one_ci(
    State(app_state): State<SharedAppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ConfigItem>, Error> {
    let configitem = configitems::load(id, &app_state.db_pool).await?;
    Ok(Json(configitem))
}

#[axum::debug_handler]
#[utoipa::path(put,
    path = "/{id}",
    request_body(
        content = ConfigItemUpdateset,
        description = "Configuration Item data to update in the database.",
        content_type = "application/json",
    ),
    responses(
        (status = OK,
            body = ConfigItem,
            description = "Configuration Item updated successfully.",
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
    tag = apidoc::CONFIG_ITEMS_TAG
)]
pub async fn update_ci(
    State(app_state): State<SharedAppState>,
    Path(id): Path<Uuid>,
    Json(configitem): Json<ConfigItemUpdateset>,
) -> Result<Json<ConfigItem>, Error> {
    let configitem = configitems::update(id, configitem, &app_state.db_pool).await?;
    Ok(Json(configitem))
}

#[axum::debug_handler]
#[utoipa::path(delete,
    path = "/{id}",
    responses(
        (status = NO_CONTENT,
            description = "Configuration Item deleted successfully.",
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
pub async fn delete_ci(
    State(app_state): State<SharedAppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, Error> {
    configitems::delete(id, &app_state.db_pool).await?;
    Ok(StatusCode::NO_CONTENT)
}
