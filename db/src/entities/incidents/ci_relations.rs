use crate::DbPool;
#[cfg(feature = "test-helpers")]
use serde::Deserialize;
use serde::Serialize;
use sqlx::Postgres;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(any(feature = "test-helpers"), derive(Deserialize))]
pub struct IncidentCIRelation {
    pub incident_id: Uuid,
    pub ci_id: Uuid,
    pub description: String,
}

/// Check if an incident with the ID sent as path param exists in the database.
async fn check_valid_incident(
    id: Uuid,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<(), crate::Error> {
    let exists = sqlx::query_scalar!(
        "
        SELECT EXISTS(SELECT 1 FROM incidents WHERE id = $1)",
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
    incident_id: Uuid,
    pool: &DbPool,
) -> Result<Vec<IncidentCIRelation>, crate::Error> {
    let mut tx = pool.begin().await?;
    check_valid_incident(incident_id, &mut *tx).await?;
    let relations = sqlx::query_as!(
        IncidentCIRelation,
        "
        SELECT incident_id, ci_id, description
        FROM incidents_ci_relations
        WHERE incident_id = $1",
        incident_id
    )
    .fetch_all(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(relations)
}

pub async fn create(
    incident_id: Uuid,
    ci_id: Uuid,
    pool: &DbPool,
) -> Result<IncidentCIRelation, crate::Error> {
    let mut tx = pool.begin().await?;
    check_valid_incident(incident_id, &mut *tx).await?;
    sqlx::query!(
        "
        INSERT INTO incidents_ci_relations (incident_id, ci_id, description)
        VALUES ($1, $2, $3)",
        incident_id,
        ci_id,
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
    Ok(IncidentCIRelation {
        incident_id: incident_id,
        ci_id: ci_id,
        description: String::from(""),
    })
}

pub async fn update(
    incident_id: Uuid,
    ci_id: Uuid,
    description: String,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<IncidentCIRelation, crate::Error> {
    match sqlx::query!(
        "
        UPDATE incidents_ci_relations
        SET description = $1
        WHERE incident_id = $2
        AND ci_id = $3
        RETURNING ci_id",
        description,
        incident_id,
        ci_id
    )
    .fetch_optional(executor)
    .await
    .map_err(crate::Error::DbError)?
    {
        Some(_) => Ok(IncidentCIRelation {
            incident_id,
            ci_id,
            description,
        }),
        None => Err(crate::Error::NoRecordFound),
    }
}

pub async fn delete(
    incident_id: Uuid,
    ci_id: Uuid,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<(), crate::Error> {
    match sqlx::query!(
        "
        DELETE FROM incidents_ci_relations
        WHERE incident_id = $1
        AND ci_id = $2
        RETURNING ci_id",
        incident_id,
        ci_id
    )
    .fetch_optional(executor)
    .await
    .map_err(crate::Error::DbError)?
    {
        Some(_) => Ok(()),
        None => Err(crate::Error::NoRecordFound),
    }
}
