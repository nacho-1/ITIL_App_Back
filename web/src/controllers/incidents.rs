use crate::{apidoc, error::Error, state::SharedAppState};
use axum::{extract::Path, extract::State, http::StatusCode, Json};
use itil_back_db::entities::incidents::{self, Incident, IncidentCreateset, IncidentUpdateset};
use tracing::info;
use uuid::Uuid;

/// Controllers for Incident-CI relations.
pub mod ci_relations;

#[axum::debug_handler]
#[utoipa::path(post,
    path = "",
    request_body(
        content = IncidentCreateset,
        description = "Incident to create in the database.",
        content_type = "application/json",
    ),
    responses(
        (status = CREATED,
            body = Incident,
            description = "Incident created successfully.",
            content_type = "application/json"
        ),
        (status = UNPROCESSABLE_ENTITY,
            description = "Request body didn't pass validations."
        ),
        (status = INTERNAL_SERVER_ERROR,
            description = "Database error."
        )
    ),
    tag = apidoc::INCIDENTS_TAG
)]
pub async fn create_incident(
    State(app_state): State<SharedAppState>,
    Json(createset): Json<IncidentCreateset>,
) -> Result<(StatusCode, Json<Incident>), Error> {
    let incident = incidents::create(createset, &app_state.db_pool).await?;
    Ok((StatusCode::CREATED, Json(incident)))
}

#[axum::debug_handler]
#[utoipa::path(get,
    path = "",
    responses(
        (status = OK,
            body = Vec<Incident>,
            description = "List of Incidents."
        ),
        (status = INTERNAL_SERVER_ERROR,
            description = "Database error."
        )
    ),
    tag = apidoc::INCIDENTS_TAG
)]
pub async fn read_all_incidents(
    State(app_state): State<SharedAppState>,
) -> Result<Json<Vec<Incident>>, Error> {
    let incidents = incidents::load_all(&app_state.db_pool).await?;

    info!("responding with {:?}", incidents);

    Ok(Json(incidents))
}

#[axum::debug_handler]
#[utoipa::path(get,
    path = "/{id}",
    responses(
        (status = OK,
            body = Incident,
            description = "OK"
        ),
        (status = NOT_FOUND,
            description = "Record not found in database."
        ),
        (status = INTERNAL_SERVER_ERROR,
            description = "Database error."
        )
    ),
    tag = apidoc::INCIDENTS_TAG
)]
pub async fn read_one_incident(
    State(app_state): State<SharedAppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Incident>, Error> {
    let incident = incidents::load(id, &app_state.db_pool).await?;
    Ok(Json(incident))
}

#[axum::debug_handler]
#[utoipa::path(put,
    path = "/{id}",
    request_body(
        content = IncidentUpdateset,
        description = "Incident data to update in the database.",
        content_type = "application/json",
    ),
    responses(
        (status = OK,
            body = Incident,
            description = "Incident updated successfully.",
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
    tag = apidoc::INCIDENTS_TAG
)]
pub async fn update_incident(
    State(app_state): State<SharedAppState>,
    Path(id): Path<Uuid>,
    Json(updateset): Json<IncidentUpdateset>,
) -> Result<Json<Incident>, Error> {
    let incident = incidents::update(id, updateset, &app_state.db_pool).await?;
    Ok(Json(incident))
}

#[axum::debug_handler]
#[utoipa::path(delete,
    path = "/{id}",
    responses(
        (status = NO_CONTENT,
            description = "Incident deleted successfully.",
        ),
        (status = NOT_FOUND,
            description = "Record not found in database."
        ),
        (status = INTERNAL_SERVER_ERROR,
            description = "Database error."
        )
    ),
    tag = apidoc::INCIDENTS_TAG
)]
pub async fn delete_incident(
    State(app_state): State<SharedAppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, Error> {
    incidents::delete(id, &app_state.db_pool).await?;
    Ok(StatusCode::NO_CONTENT)
}
