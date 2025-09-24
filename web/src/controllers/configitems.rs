use crate::{error::Error, state::SharedAppState};
use axum::{extract::Path, extract::State, http::StatusCode, Json};
use itil_back_db::entities;
use tracing::info;
use uuid::Uuid;

#[axum::debug_handler]
pub async fn create(
    State(app_state): State<SharedAppState>,
    Json(configitem): Json<entities::configitems::ConfigItemChangeset>,
) -> Result<(StatusCode, Json<entities::configitems::ConfigItem>), Error> {
    let configitem = entities::configitems::create(configitem, &app_state.db_pool).await?;
    Ok((StatusCode::CREATED, Json(configitem)))
}

#[axum::debug_handler]
pub async fn read_all(
    State(app_state): State<SharedAppState>,
) -> Result<Json<Vec<entities::configitems::ConfigItem>>, Error> {
    let configitems = entities::configitems::load_all(&app_state.db_pool).await?;

    info!("responding with {:?}", configitems);

    Ok(Json(configitems))
}

#[axum::debug_handler]
pub async fn read_one(
    State(app_state): State<SharedAppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<entities::configitems::ConfigItem>, Error> {
    let configitem = entities::configitems::load(id, &app_state.db_pool).await?;
    Ok(Json(configitem))
}

#[axum::debug_handler]
pub async fn update(
    State(app_state): State<SharedAppState>,
    Path(id): Path<Uuid>,
    Json(configitem): Json<entities::configitems::ConfigItemChangeset>,
) -> Result<Json<entities::configitems::ConfigItem>, Error> {
    let configitem = entities::configitems::update(id, configitem, &app_state.db_pool).await?;
    Ok(Json(configitem))
}

#[axum::debug_handler]
pub async fn delete(
    State(app_state): State<SharedAppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, Error> {
    entities::configitems::delete(id, &app_state.db_pool).await?;
    Ok(StatusCode::NO_CONTENT)
}
