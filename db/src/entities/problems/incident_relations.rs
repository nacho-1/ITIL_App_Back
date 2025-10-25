use crate::DbPool;
#[cfg(feature = "test-helpers")]
use serde::Deserialize;
use serde::Serialize;
use sqlx::Postgres;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(any(feature = "test-helpers"), derive(Deserialize))]
pub struct ProblemIncidentRelation {
    pub problem_id: Uuid,
    pub incident_id: Uuid,
    #[schema(example = "Is caused by")]
    pub description: String,
}

/// Check if a problem with the ID sent as path param exists in the database.
async fn check_valid_problem(
    id: Uuid,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<(), crate::Error> {
    let exists = sqlx::query_scalar!(
        "
        SELECT EXISTS(SELECT 1 FROM problems WHERE id = $1)",
        id
    )
    .fetch_one(executor)
    .await?;

    if !exists.unwrap_or(false) {
        return Err(crate::Error::NoRecordFound);
    }

    Ok(())
}

pub async fn load_all(
    problem_id: Uuid,
    pool: &DbPool,
) -> Result<Vec<ProblemIncidentRelation>, crate::Error> {
    let mut tx = pool.begin().await?;
    check_valid_problem(problem_id, &mut *tx).await?;
    let relations = sqlx::query_as!(
        ProblemIncidentRelation,
        "
        SELECT problem_id, incident_id, description
        FROM problem_incident_relations
        WHERE problem_id = $1",
        problem_id
    )
    .fetch_all(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(relations)
}

pub async fn create(
    problem_id: Uuid,
    incident_id: Uuid,
    pool: &DbPool,
) -> Result<ProblemIncidentRelation, crate::Error> {
    let mut tx = pool.begin().await?;
    check_valid_problem(problem_id, &mut *tx).await?;
    sqlx::query!(
        "
        INSERT INTO problem_incident_relations (problem_id, incident_id, description)
        VALUES ($1, $2, $3)",
        problem_id,
        incident_id,
        String::from("")
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref dbe) if dbe.is_foreign_key_violation() => {
            crate::Error::ConstraintError
        }
        _ => crate::Error::DbError(e),
    })?;

    tx.commit().await?;
    Ok(ProblemIncidentRelation {
        problem_id,
        incident_id,
        description: String::from(""),
    })
}

pub async fn update(
    problem_id: Uuid,
    incident_id: Uuid,
    description: String,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<ProblemIncidentRelation, crate::Error> {
    match sqlx::query_as!(
        ProblemIncidentRelation,
        "
        UPDATE problem_incident_relations
        SET description = $1
        WHERE problem_id = $2
        AND incident_id = $3
        RETURNING problem_id, incident_id, description",
        description,
        problem_id,
        incident_id
    )
    .fetch_optional(executor)
    .await
    .map_err(crate::Error::DbError)?
    {
        Some(updated_relation) => Ok(updated_relation),
        None => Err(crate::Error::NoRecordFound),
    }
}

pub async fn delete(
    problem_id: Uuid,
    incident_id: Uuid,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<(), crate::Error> {
    match sqlx::query!(
        "
        DELETE FROM problem_incident_relations
        WHERE problem_id = $1
        AND incident_id = $2
        RETURNING incident_id",
        problem_id,
        incident_id
    )
    .fetch_optional(executor)
    .await
    .map_err(crate::Error::DbError)?
    {
        Some(_) => Ok(()),
        None => Err(crate::Error::NoRecordFound),
    }
}
