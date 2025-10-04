use crate::{apidoc, error::Error, state::SharedAppState};
use axum::{extract::Path, extract::State, http::StatusCode, Json};
use itil_back_db::entities;
use tracing::info;
use uuid::Uuid;

#[axum::debug_handler]
#[utoipa::path(post,
    path = "",
    request_body(
        content = entities::incidents::IncidentChangeset,
        description = "Incident to create in the database",
        content_type = "application/json",
    ),
    responses(
        (status = CREATED,
            body = entities::incidents::Incident,
            description = "Incident created successfully",
            content_type = "application/json"
        ),
        (status = UNPROCESSABLE_ENTITY,
            description = "Request body didn't pass validations"
        ),
        (status = INTERNAL_SERVER_ERROR,
            description = "Database error"
        )
    ),
    tag = apidoc::INCIDENTS_TAG
)]
pub async fn create_incident(
    State(app_state): State<SharedAppState>,
    Json(incident): Json<entities::incidents::IncidentChangeset>,
) -> Result<(StatusCode, Json<entities::incidents::Incident>), Error> {
    let incident = entities::incidents::create(incident, &app_state.db_pool).await?;
    Ok((StatusCode::CREATED, Json(incident)))
}

#[axum::debug_handler]
#[utoipa::path(get,
    path = "",
    responses(
        (status = OK,
            body = Vec<entities::incidents::Incident>,
            description = "List of Incidents"
        ),
        (status = INTERNAL_SERVER_ERROR,
            description = "Database error"
        )
    ),
    tag = apidoc::INCIDENTS_TAG
)]
pub async fn read_all_incidents(
    State(app_state): State<SharedAppState>,
) -> Result<Json<Vec<entities::incidents::Incident>>, Error> {
    let incidents = entities::incidents::load_all(&app_state.db_pool).await?;

    info!("responding with {:?}", incidents);

    Ok(Json(incidents))
}

#[axum::debug_handler]
#[utoipa::path(get,
    path = "/{id}",
    responses(
        (status = OK,
            body = entities::incidents::Incident,
            description = "Successful operation"
        ),
        (status = NOT_FOUND,
            description = "Record not found in database"
        ),
        (status = INTERNAL_SERVER_ERROR,
            description = "Database error"
        )
    ),
    tag = apidoc::INCIDENTS_TAG
)]
pub async fn read_one_incident(
    State(app_state): State<SharedAppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<entities::incidents::Incident>, Error> {
    let incident = entities::incidents::load(id, &app_state.db_pool).await?;
    Ok(Json(incident))
}

#[axum::debug_handler]
#[utoipa::path(put,
    path = "/{id}",
    request_body(
        content = entities::incidents::IncidentChangeset,
        description = "Incident data to update in the database",
        content_type = "application/json",
    ),
    responses(
        (status = OK,
            body = entities::incidents::Incident,
            description = "Incident updated successfully",
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
    tag = apidoc::INCIDENTS_TAG
)]
pub async fn update_incident(
    State(app_state): State<SharedAppState>,
    Path(id): Path<Uuid>,
    Json(incident): Json<entities::incidents::IncidentChangeset>,
) -> Result<Json<entities::incidents::Incident>, Error> {
    let incident = entities::incidents::update(id, incident, &app_state.db_pool).await?;
    Ok(Json(incident))
}

#[axum::debug_handler]
#[utoipa::path(delete,
    path = "/{id}",
    responses(
        (status = NO_CONTENT,
            description = "Incident deleted successfully",
        ),
        (status = NOT_FOUND,
            description = "Record not found in database"
        ),
        (status = INTERNAL_SERVER_ERROR,
            description = "Database error"
        )
    ),
    tag = apidoc::INCIDENTS_TAG
)]
pub async fn delete_incident(
    State(app_state): State<SharedAppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, Error> {
    entities::incidents::delete(id, &app_state.db_pool).await?;
    Ok(StatusCode::NO_CONTENT)
}
