use crate::{apidoc, error::Error, state::SharedAppState};
use axum::{extract::Path, extract::State, http::StatusCode, Json};
use itil_back_db::entities::problems::{self, Problem, ProblemCreateset, ProblemUpdateset};
use tracing::info;
use uuid::Uuid;

pub mod incident_relations;

#[axum::debug_handler]
#[utoipa::path(post,
    path = "",
    request_body(
        content = ProblemCreateset,
        description = "Problem to create in the database.",
        content_type = "application/json",
    ),
    responses(
        (status = CREATED,
            body = Problem,
            description = "Problem created successfully.",
            content_type = "application/json"
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
pub async fn create_problem(
    State(app_state): State<SharedAppState>,
    Json(problem): Json<ProblemCreateset>,
) -> Result<(StatusCode, Json<Problem>), Error> {
    let problem = problems::create(problem, &app_state.db_pool).await?;
    Ok((StatusCode::CREATED, Json(problem)))
}

#[axum::debug_handler]
#[utoipa::path(get,
    path = "",
    responses(
        (status = OK,
            body = Vec<Problem>,
            description = "List of Problems."
        ),
        (status = INTERNAL_SERVER_ERROR,
            description = "Database error."
        )
    ),
    tag = apidoc::PROBLEMS_TAG
)]
pub async fn read_all_problems(
    State(app_state): State<SharedAppState>,
) -> Result<Json<Vec<Problem>>, Error> {
    let problems = problems::load_all(&app_state.db_pool).await?;

    info!("responding with {:?}", problems);

    Ok(Json(problems))
}

#[axum::debug_handler]
#[utoipa::path(get,
    path = "/{id}",
    responses(
        (status = OK,
            body = Problem,
            description = "OK"
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
pub async fn read_one_problem(
    State(app_state): State<SharedAppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Problem>, Error> {
    let problem = problems::load(id, &app_state.db_pool).await?;
    Ok(Json(problem))
}

#[axum::debug_handler]
#[utoipa::path(put,
    path = "/{id}",
    request_body(
        content = ProblemUpdateset,
        description = "Problem data to update in the database.",
        content_type = "application/json",
    ),
    responses(
        (status = OK,
            body = Problem,
            description = "Problem updated successfully.",
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
    tag = apidoc::PROBLEMS_TAG
)]
pub async fn update_problem(
    State(app_state): State<SharedAppState>,
    Path(id): Path<Uuid>,
    Json(problem): Json<ProblemUpdateset>,
) -> Result<Json<Problem>, Error> {
    let problem = problems::update(id, problem, &app_state.db_pool).await?;
    Ok(Json(problem))
}

#[axum::debug_handler]
#[utoipa::path(delete,
    path = "/{id}",
    responses(
        (status = NO_CONTENT,
            description = "Problem deleted successfully.",
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
pub async fn delete_problem(
    State(app_state): State<SharedAppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, Error> {
    problems::delete(id, &app_state.db_pool).await?;
    Ok(StatusCode::NO_CONTENT)
}
