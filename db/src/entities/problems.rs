use crate::entity_helpers::PatchField;
use serde::Deserialize;
use serde::Serialize;
use sqlx::types::chrono::DateTime;
use sqlx::types::chrono::Utc;
use sqlx::Postgres;
use sqlx::Type;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Serialize, Debug, ToSchema)]
#[cfg_attr(any(feature = "test-helpers", test), derive(Deserialize))]
pub struct Problem {
    pub id: Uuid,
    #[schema(example = "Low Bandwidth in Office")]
    pub title: String,
    pub status: ProblemStatus,
    pub detection_timedate: DateTime<Utc>,
    #[schema(
        example = "Bandwidth doesn't go above 1Mb/s on devices connected to the main office network."
    )]
    pub description: String,
    #[schema(example = "Router is misconfigured and doesn't do hardware offloading.")]
    pub causes: String,
    #[schema(example = "docs.local/workarounds/003.pdf")]
    pub workarounds: Option<String>,
    #[schema(example = "docs.local/resolutions/002.pdf")]
    pub resolutions: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Type, Debug, ToSchema)]
#[sqlx(type_name = "problem_status", rename_all = "lowercase")]
#[schema(example = "resolved")]
#[serde(rename_all = "lowercase")]
#[cfg_attr(any(feature = "test-helpers", test), derive(PartialEq))]
pub enum ProblemStatus {
    Open,
    KnownError,
    Resolved,
    Closed,
}

#[derive(Deserialize, Validate, Clone, ToSchema)]
#[cfg_attr(feature = "test-helpers", derive(Serialize))]
pub struct ProblemCreateset {
    #[schema(example = "Low Bandwidth in Office")]
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    pub status: Option<ProblemStatus>,
    pub detection_timedate: Option<DateTime<Utc>>,
    #[schema(
        example = "Bandwidth doesn't go above 1Mb/s on devices connected to the main office network."
    )]
    #[validate(length(max = 1024))]
    pub description: String,
    #[schema(example = "Router is misconfigured and doesn't do hardware offloading.")]
    #[validate(length(max = 1024))]
    pub causes: String,
    #[schema(example = "docs.local/workarounds/003.pdf")]
    #[validate(length(max = 1024))]
    pub workarounds: Option<String>,
    #[schema(example = "docs.local/resolutions/002.pdf")]
    #[validate(length(max = 1024))]
    pub resolutions: Option<String>,
}

#[derive(Deserialize, Validate, Clone, ToSchema)]
#[cfg_attr(feature = "test-helpers", derive(Serialize))]
pub struct ProblemUpdateset {
    #[schema(example = "Low Bandwidth in Office")]
    #[validate(length(min = 1, max = 255))]
    pub title: Option<String>,
    pub status: Option<ProblemStatus>,
    pub detection_timedate: Option<DateTime<Utc>>,
    #[schema(
        example = "Bandwidth doesn't go above 1Mb/s on devices connected to the main office network."
    )]
    #[validate(length(max = 1024))]
    pub description: Option<String>,
    #[schema(example = "Router is misconfigured and doesn't do hardware offloading.")]
    #[validate(length(max = 1024))]
    pub causes: Option<String>,
    #[schema(example = "docs.local/workarounds/003.pdf")]
    #[validate(length(max = 1024))]
    #[serde(default)]
    #[cfg_attr(
        any(feature = "test-helpers", test),
        serde(skip_serializing_if = "PatchField::leave_unchanged")
    )]
    pub workarounds: PatchField<String>,
    #[schema(example = "docs.local/resolutions/002.pdf")]
    #[validate(length(max = 1024))]
    #[serde(default)]
    #[cfg_attr(
        any(feature = "test-helpers", test),
        serde(skip_serializing_if = "PatchField::leave_unchanged")
    )]
    pub resolutions: PatchField<String>,
}

pub async fn load_all(
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<Vec<Problem>, crate::Error> {
    let problems = sqlx::query_as!(
        Problem,
        "
        SELECT id, title, status as \"status: ProblemStatus\", detection_timedate,
            description, causes, workarounds, resolutions
        FROM problems"
    )
    .fetch_all(executor)
    .await?;

    Ok(problems)
}

pub async fn load(
    id: Uuid,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<Problem, crate::Error> {
    match sqlx::query_as!(
        Problem,
        "
        SELECT id, title, status as \"status: ProblemStatus\", detection_timedate,
            description, causes, workarounds, resolutions
        FROM problems
        WHERE id = $1",
        id
    )
    .fetch_optional(executor)
    .await
    .map_err(crate::Error::DbError)?
    {
        Some(problem) => Ok(problem),
        None => Err(crate::Error::NoRecordFound),
    }
}

pub async fn create(
    problem: ProblemCreateset,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<Problem, crate::Error> {
    problem.validate()?;

    let created_problem = sqlx::query_as!(
        Problem,
        "
        INSERT INTO problems (title, status, detection_timedate,
            description, causes, workarounds, resolutions)
        VALUES ($1, $2, COALESCE($3, now()), $4, $5, $6, $7)
        RETURNING id, title, status as \"status: ProblemStatus\", detection_timedate,
            description, causes, workarounds, resolutions",
        problem.title,
        problem.status.unwrap_or(ProblemStatus::Open) as ProblemStatus,
        problem.detection_timedate,
        problem.description,
        problem.causes,
        problem.workarounds,
        problem.resolutions,
    )
    .fetch_one(executor)
    .await
    .map_err(crate::Error::DbError)?;

    Ok(created_problem)
}

pub async fn update(
    id: Uuid,
    problem: ProblemUpdateset,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<Problem, crate::Error> {
    problem.validate()?;

    match sqlx::query_as!(
        Problem,
        "
        UPDATE problems
        SET title = COALESCE($1, title), status = COALESCE($2, status),
            detection_timedate = COALESCE($3, detection_timedate),
            description = COALESCE($4, description), causes = COALESCE($5, causes),
            workarounds = CASE
                WHEN $6 THEN workarounds
                ELSE $7
            END,
            resolutions = CASE
                WHEN $8 THEN resolutions
                ELSE $9
            END
        WHERE id = $10
        RETURNING id, title, status as \"status: ProblemStatus\", detection_timedate,
            description, causes, workarounds, resolutions",
        problem.title,                                     // 1
        problem.status as Option<ProblemStatus>,           // 2
        problem.detection_timedate,                        // 3
        problem.description,                               // 4
        problem.causes,                                    // 5
        problem.workarounds.leave_unchanged(),             // 6
        Into::<Option<String>>::into(problem.workarounds), // 7
        problem.resolutions.leave_unchanged(),             // 8
        Into::<Option<String>>::into(problem.resolutions), // 9
        id,                                                // 10
    )
    .fetch_optional(executor)
    .await
    .map_err(crate::Error::DbError)?
    {
        Some(updated_problem) => Ok(updated_problem),
        None => Err(crate::Error::NoRecordFound),
    }
}

pub async fn delete(
    id: Uuid,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<(), crate::Error> {
    match sqlx::query!(
        "
        DELETE FROM problems
        WHERE id = $1
        RETURNING id",
        id
    )
    .fetch_optional(executor)
    .await
    .map_err(crate::Error::DbError)?
    {
        Some(_) => Ok(()),
        None => Err(crate::Error::NoRecordFound),
    }
}
