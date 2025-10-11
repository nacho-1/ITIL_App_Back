use crate::{apidoc, error::Error, state::SharedAppState};
use axum::{extract::Path, extract::State, http::StatusCode, Json};
use itil_back_db::entities::incidents::ci_relations::{self, IncidentCIRelation};
use serde::Deserialize;
#[cfg(feature = "test-helpers")]
use serde::Serialize;
use tracing::info;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, ToSchema)]
#[cfg_attr(feature = "test-helpers", derive(Serialize))]
pub struct RelateCIRequest {
    pub ci_id: Uuid,
}

#[derive(Deserialize, ToSchema)]
#[cfg_attr(feature = "test-helpers", derive(Serialize))]
pub struct ModifyIncidentCIRelation {
    pub description: String,
}

#[axum::debug_handler]
#[utoipa::path(post,
    path = "/{id}/configitems",
    request_body(
        content = RelateCIRequest,
        description = "Configuration Item info necessary for linking.",
        content_type = "application/json",
    ),
    responses(
        (status = CREATED,
            body = IncidentCIRelation,
            description = "Relation created successfully.",
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
    tag = apidoc::INCIDENTS_TAG
)]
pub async fn create_incident_ci_relation(
    State(app_state): State<SharedAppState>,
    Path(incident_id): Path<Uuid>,
    Json(request): Json<RelateCIRequest>,
) -> Result<(StatusCode, Json<IncidentCIRelation>), Error> {
    let relation = ci_relations::create(incident_id, request.ci_id, &app_state.db_pool).await?;
    Ok((StatusCode::CREATED, Json(relation)))
}

#[axum::debug_handler]
#[utoipa::path(get,
    path = "/{id}/configitems",
    responses(
        (status = OK,
            body = Vec<IncidentCIRelation>,
            description = "List of related CIs."
        ),
        (status = NOT_FOUND,
            description = "Resource doesn't exist."
        ),
        (status = INTERNAL_SERVER_ERROR,
            description = "Database error."
        )
    ),
    tag = apidoc::INCIDENTS_TAG
)]
pub async fn read_all_incident_ci_relations(
    State(app_state): State<SharedAppState>,
    Path(incidet_id): Path<Uuid>,
) -> Result<Json<Vec<IncidentCIRelation>>, Error> {
    let relations = ci_relations::load_all(incidet_id, &app_state.db_pool).await?;

    info!("responding with {:?}", relations);

    Ok(Json(relations))
}

#[axum::debug_handler]
#[utoipa::path(put,
    path = "/{id}/configitems/{ci_id}",
    request_body(
        content = ModifyIncidentCIRelation,
        description = "Relation data to update in the database.",
        content_type = "application/json",
    ),
    responses(
        (status = OK,
            body = IncidentCIRelation,
            description = "Relation updated successfully.",
            content_type = "application/json"
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
pub async fn update_incident_ci_relation(
    State(app_state): State<SharedAppState>,
    Path((incident_id, ci_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<ModifyIncidentCIRelation>,
) -> Result<Json<IncidentCIRelation>, Error> {
    let relation =
        ci_relations::update(incident_id, ci_id, request.description, &app_state.db_pool).await?;
    Ok(Json(relation))
}

#[axum::debug_handler]
#[utoipa::path(delete,
    path = "/{id}/configitems/{ci_id}",
    responses(
        (status = NO_CONTENT,
            description = "Relation deleted successfully.",
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
pub async fn delete_incident_ci_relation(
    State(app_state): State<SharedAppState>,
    Path((incident_id, ci_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, Error> {
    ci_relations::delete(incident_id, ci_id, &app_state.db_pool).await?;
    Ok(StatusCode::NO_CONTENT)
}
