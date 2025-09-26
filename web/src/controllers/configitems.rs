use crate::{apidoc, error::Error, state::SharedAppState};
use axum::{extract::Path, extract::State, http::StatusCode, Json};
use itil_back_db::entities;
use tracing::info;
use uuid::Uuid;

#[axum::debug_handler]
#[utoipa::path(post, 
    path = "", 
    request_body(
        content = entities::configitems::ConfigItemChangeset,
        description = "Configuration Item to create in the database",
        content_type = "application/json",
    ), 
    responses(
        (status = CREATED, 
            body = entities::configitems::ConfigItem,
            description = "Configuration Item created successfully",
            content_type = "application/json"
        ),
        (status = UNPROCESSABLE_ENTITY,
            description = "Request body didn't pass validations"
        ),
        (status = INTERNAL_SERVER_ERROR,
            description = "Database error"
        )
    ), 
    tag = apidoc::CONFIG_ITEMS_TAG
)]
pub async fn create(
    State(app_state): State<SharedAppState>,
    Json(configitem): Json<entities::configitems::ConfigItemChangeset>,
) -> Result<(StatusCode, Json<entities::configitems::ConfigItem>), Error> {
    let configitem = entities::configitems::create(configitem, &app_state.db_pool).await?;
    Ok((StatusCode::CREATED, Json(configitem)))
}

#[axum::debug_handler]
#[utoipa::path(get,
    path = "",
    responses(
        (status = OK,
            body = Vec<entities::configitems::ConfigItem>,
            description = "List of Configuration Items"
        ),
        (status = INTERNAL_SERVER_ERROR,
            description = "Database error"
        )
    ),
    tag = apidoc::CONFIG_ITEMS_TAG
)]
pub async fn read_all(
    State(app_state): State<SharedAppState>,
) -> Result<Json<Vec<entities::configitems::ConfigItem>>, Error> {
    let configitems = entities::configitems::load_all(&app_state.db_pool).await?;

    info!("responding with {:?}", configitems);

    Ok(Json(configitems))
}

#[axum::debug_handler]
#[utoipa::path(get,
    path = "/{id}",
    responses(
        (status = OK,
            body = entities::configitems::ConfigItem,
            description = "Configuration Item"
        ),
        (status = NOT_FOUND,
            description = "Record not found in database"
        ),
        (status = INTERNAL_SERVER_ERROR,
            description = "Database error"
        )
    ),
    tag = apidoc::CONFIG_ITEMS_TAG
)]
pub async fn read_one(
    State(app_state): State<SharedAppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<entities::configitems::ConfigItem>, Error> {
    let configitem = entities::configitems::load(id, &app_state.db_pool).await?;
    Ok(Json(configitem))
}

#[axum::debug_handler]
#[utoipa::path(put, 
    path = "/{id}", 
    request_body(
        content = entities::configitems::ConfigItemChangeset,
        description = "Configuration Item data to update in the database",
        content_type = "application/json",
    ), 
    responses(
        (status = OK, 
            body = entities::configitems::ConfigItem,
            description = "Configuration Item updated successfully",
            content_type = "application/json"
        ),
        (status = UNPROCESSABLE_ENTITY,
            description = "Request body didn't pass validations"
        ),
        (status = NOT_FOUND,
            description = "Record not found in database"
        ),
        (status = INTERNAL_SERVER_ERROR,
            description = "Database error"
        )
    ), 
    tag = apidoc::CONFIG_ITEMS_TAG
)]
pub async fn update(
    State(app_state): State<SharedAppState>,
    Path(id): Path<Uuid>,
    Json(configitem): Json<entities::configitems::ConfigItemChangeset>,
) -> Result<Json<entities::configitems::ConfigItem>, Error> {
    let configitem = entities::configitems::update(id, configitem, &app_state.db_pool).await?;
    Ok(Json(configitem))
}

#[axum::debug_handler]
#[utoipa::path(delete, 
    path = "/{id}", 
    responses(
        (status = NO_CONTENT, 
            description = "Configuration Item deleted successfully",
        ),
        (status = NOT_FOUND,
            description = "Record not found in database"
        ),
        (status = INTERNAL_SERVER_ERROR,
            description = "Database error"
        )
    ), 
    tag = apidoc::CONFIG_ITEMS_TAG
)]
pub async fn delete(
    State(app_state): State<SharedAppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, Error> {
    entities::configitems::delete(id, &app_state.db_pool).await?;
    Ok(StatusCode::NO_CONTENT)
}
