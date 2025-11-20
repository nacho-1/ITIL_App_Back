use crate::{apidoc, error::Error, state::SharedAppState};
use axum::{extract::Path, extract::State, http::StatusCode, Json};
use itil_back_db::entities::changes::problem_relations::{
    self, RFCProblemCreateset, RFCProblemRelation,
};
use tracing::info;
use uuid::Uuid;

#[axum::debug_handler]
#[utoipa::path(post,
    path = "/{id}/problems",
    request_body(
        content = RFCProblemCreateset,
        description = "Relation info necessary for linking.",
        content_type = "application/json",
    ),
    responses(
        (status = CREATED,
            body = RFCProblemRelation,
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
    tag = apidoc::CHANGES_TAG
)]
pub async fn create_rfc_problem_relation(
    State(app_state): State<SharedAppState>,
    Path(rfc_id): Path<Uuid>,
    Json(createset): Json<RFCProblemCreateset>,
) -> Result<(StatusCode, Json<RFCProblemRelation>), Error> {
    let relation = problem_relations::create(rfc_id, createset, &app_state.db_pool).await?;
    Ok((StatusCode::CREATED, Json(relation)))
}

#[axum::debug_handler]
#[utoipa::path(get,
    path = "/{id}/problems",
    responses(
        (status = OK,
            body = Vec<RFCProblemRelation>,
            description = "List of relations."
        ),
        (status = NOT_FOUND,
            description = "Resource doesn't exist."
        ),
        (status = INTERNAL_SERVER_ERROR,
            description = "Database error."
        )
    ),
    tag = apidoc::CHANGES_TAG
)]
pub async fn read_all_rfc_problem_relations(
    State(app_state): State<SharedAppState>,
    Path(rfc_id): Path<Uuid>,
) -> Result<Json<Vec<RFCProblemRelation>>, Error> {
    let changes = problem_relations::load_all(rfc_id, &app_state.db_pool).await?;

    info!("responding with {:?}", changes);

    Ok(Json(changes))
}

#[axum::debug_handler]
#[utoipa::path(delete,
    path = "/{id}/problems/{relation_id}",
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
    tag = apidoc::CHANGES_TAG
)]
pub async fn delete_rfc_problem_relation(
    State(app_state): State<SharedAppState>,
    Path((rfc_id, relation_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, Error> {
    problem_relations::delete(rfc_id, relation_id, &app_state.db_pool).await?;
    Ok(StatusCode::NO_CONTENT)
}
