use crate::DbPool;
use serde::Deserialize;
use serde::Serialize;
use sqlx::Postgres;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(any(feature = "test-helpers"), derive(Deserialize, PartialEq))]
pub struct RFCProblemRelation {
    pub id: Uuid,
    pub rfc_id: Uuid,
    pub problem_id: Uuid,
}

#[derive(Clone, Deserialize, ToSchema, Validate)]
#[cfg_attr(any(feature = "test-helpers"), derive(Serialize))]
pub struct RFCProblemCreateset {
    pub problem_id: Uuid,
}

async fn check_valid_rfc(
    id: Uuid,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<(), crate::Error> {
    let exists = sqlx::query_scalar!(
        "
        SELECT EXISTS(SELECT 1 FROM rfcs WHERE id = $1)",
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
    rfc_id: Uuid,
    pool: &DbPool,
) -> Result<Vec<RFCProblemRelation>, crate::Error> {
    let mut tx = pool.begin().await?;
    check_valid_rfc(rfc_id, &mut *tx).await?;
    let relations = sqlx::query_as!(
        RFCProblemRelation,
        "
        SELECT id, rfc_id, problem_id
        FROM rfc_problem_relations
        WHERE rfc_id = $1",
        rfc_id
    )
    .fetch_all(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(relations)
}

pub async fn create(
    rfc_id: Uuid,
    createset: RFCProblemCreateset,
    pool: &DbPool,
) -> Result<RFCProblemRelation, crate::Error> {
    createset.validate()?;
    let mut tx = pool.begin().await?;
    check_valid_rfc(rfc_id, &mut *tx).await?;
    let created_relation = sqlx::query_as!(
        RFCProblemRelation,
        "
        INSERT INTO rfc_problem_relations (rfc_id, problem_id)
        VALUES ($1, $2)
        RETURNING id, rfc_id, problem_id",
        rfc_id,
        createset.problem_id,
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref dbe) if dbe.is_foreign_key_violation() => {
            crate::Error::ConstraintError
        }
        _ => crate::Error::DbError(e),
    })?;

    tx.commit().await?;
    Ok(created_relation)
}

pub async fn delete(
    rfc_id: Uuid,
    id: Uuid,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<(), crate::Error> {
    match sqlx::query!(
        "
        DELETE FROM rfc_problem_relations
        WHERE rfc_id = $1
        AND id = $2
        RETURNING id",
        rfc_id,
        id,
    )
    .fetch_optional(executor)
    .await
    .map_err(crate::Error::DbError)?
    {
        Some(_) => Ok(()),
        None => Err(crate::Error::NoRecordFound),
    }
}
