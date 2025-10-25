use crate::{apidoc, error::Error, state::SharedAppState};
use axum::{extract::Path, extract::State, http::StatusCode, Json};
use itil_back_db::entities::problems::incident_relations::{self, ProblemIncidentRelation};
use serde::Deserialize;
#[cfg(feature = "test-helpers")]
use serde::Serialize;
use tracing::info;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Deserialize, ToSchema)]
#[cfg_attr(feature = "test-helpers", derive(Serialize))]
pub struct CreateIncidentRelation {
    pub incident_id: Uuid,
}

#[derive(Deserialize, ToSchema)]
#[cfg_attr(feature = "test-helpers", derive(Serialize))]
pub struct UpdateIncidentRelation {
    pub description: String,
}

#[axum::debug_handler]
#[utoipa::path(post,
    path = "/{id}/incidents",
    request_body(
        content = CreateIncidentRelation,
        description = "Incident info necessary for linking.",
        content_type = "application/json",
    ),
    responses(
        (status = CREATED,
            body = ProblemIncidentRelation,
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
    tag = apidoc::PROBLEMS_TAG
)]
pub async fn create_problem_incident_relation(
    State(app_state): State<SharedAppState>,
    Path(problem_id): Path<Uuid>,
    Json(request): Json<CreateIncidentRelation>,
) -> Result<(StatusCode, Json<ProblemIncidentRelation>), Error> {
    let relation =
        incident_relations::create(problem_id, request.incident_id, &app_state.db_pool).await?;
    Ok((StatusCode::CREATED, Json(relation)))
}

#[axum::debug_handler]
#[utoipa::path(get,
    path = "/{id}/incidents",
    responses(
        (status = OK,
            body = Vec<ProblemIncidentRelation>,
            description = "List of related Incidents."
        ),
        (status = NOT_FOUND,
            description = "Resource doesn't exist."
        ),
        (status = INTERNAL_SERVER_ERROR,
            description = "Database error."
        )
    ),
    tag = apidoc::PROBLEMS_TAG
)]
pub async fn read_all_problem_incident_relations(
    State(app_state): State<SharedAppState>,
    Path(problem_id): Path<Uuid>,
) -> Result<Json<Vec<ProblemIncidentRelation>>, Error> {
    let relations = incident_relations::load_all(problem_id, &app_state.db_pool).await?;

    info!("responding with {:?}", relations);

    Ok(Json(relations))
}

#[axum::debug_handler]
#[utoipa::path(put,
    path = "/{id}/incidents/{incident_id}",
    request_body(
        content = UpdateIncidentRelation,
        description = "Relation data to update in the database.",
        content_type = "application/json",
    ),
    responses(
        (status = OK,
            body = ProblemIncidentRelation,
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
    tag = apidoc::PROBLEMS_TAG
)]
pub async fn update_problem_incident_relation(
    State(app_state): State<SharedAppState>,
    Path((problem_id, incident_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateIncidentRelation>,
) -> Result<Json<ProblemIncidentRelation>, Error> {
    let relation = incident_relations::update(
        problem_id,
        incident_id,
        request.description,
        &app_state.db_pool,
    )
    .await?;
    Ok(Json(relation))
}

#[axum::debug_handler]
#[utoipa::path(delete,
    path = "/{id}/incidents/{incident_id}",
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
    tag = apidoc::PROBLEMS_TAG
)]
pub async fn delete_problem_incident_relation(
    State(app_state): State<SharedAppState>,
    Path((problem_id, incident_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, Error> {
    incident_relations::delete(problem_id, incident_id, &app_state.db_pool).await?;
    Ok(StatusCode::NO_CONTENT)
}
