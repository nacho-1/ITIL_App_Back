use crate::{entity_helpers, DbPool};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde::Serialize;
use sqlx::Postgres;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::{Validate, ValidationError};

#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(any(feature = "test-helpers"), derive(Deserialize))]
pub struct CIChange {
    pub id: Uuid,
    pub ci_id: Uuid,
    pub implementation_timedate: DateTime<Utc>,
    #[schema(example = "docs.local/changes/ci001/987.pdf")]
    pub documentation: String,
}

/// Payload for creating a change record.
#[derive(Clone, Deserialize, ToSchema, Validate)]
#[cfg_attr(any(feature = "test-helpers"), derive(Serialize))]
pub struct CIChangeCreateset {
    pub implementation_timedate: DateTime<Utc>,
    #[validate(length(max = 1024))]
    #[schema(example = "docs.local/changes/ci001/987.pdf")]
    pub documentation: String,
}

/// Payload for updating a change record.
#[derive(Clone, Deserialize, ToSchema, Validate)]
#[validate(schema(function = "validate_required_fields"))]
#[cfg_attr(any(feature = "test-helpers"), derive(Serialize))]
pub struct CIChangeUpdateset {
    #[serde(default, with = "::serde_with::rust::double_option")]
    #[cfg_attr(
        any(feature = "test-helpers"),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub implementation_timedate: Option<Option<DateTime<Utc>>>,
    #[schema(example = "docs.local/changes/ci001/987.pdf")]
    #[validate(length(max = 1024))]
    #[serde(default, with = "::serde_with::rust::double_option")]
    #[cfg_attr(
        any(feature = "test-helpers"),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub documentation: Option<Option<String>>,
}

/// Validate that required fields of [CIChangeUpdateset] aren't explicitly null.
fn validate_required_fields(updateset: &CIChangeUpdateset) -> Result<(), ValidationError> {
    entity_helpers::validate_not_null(&updateset.implementation_timedate)?;
    entity_helpers::validate_not_null(&updateset.documentation)?;

    Ok(())
}

/// Check if a configuration item with the ID sent as path param exists in the database.
async fn check_valid_ci(
    id: Uuid,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<(), crate::Error> {
    let exists = sqlx::query_scalar!(
        "
        SELECT EXISTS(SELECT 1 FROM configitems WHERE id = $1)",
        id
    )
    .fetch_one(executor)
    .await?;

    if !exists.unwrap_or(false) {
        return Err(crate::Error::NoRecordFound);
    }

    Ok(())
}

pub async fn load_all(ci_id: Uuid, pool: &DbPool) -> Result<Vec<CIChange>, crate::Error> {
    let mut tx = pool.begin().await?;
    check_valid_ci(ci_id, &mut *tx).await?;
    let changes = sqlx::query_as!(
        CIChange,
        "
        SELECT id, ci_id, implementation_timedate, documentation
        FROM ci_changes
        WHERE ci_id = $1
        ORDER BY implementation_timedate DESC",
        ci_id
    )
    .fetch_all(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(changes)
}

pub async fn load(
    id: Uuid,
    ci_id: Uuid,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<CIChange, crate::Error> {
    match sqlx::query_as!(
        CIChange,
        "
        SELECT id, ci_id, implementation_timedate, documentation
        FROM ci_changes
        WHERE id = $1
        AND ci_id = $2",
        id,
        ci_id
    )
    .fetch_optional(executor)
    .await
    .map_err(crate::Error::DbError)?
    {
        Some(change) => Ok(change),
        None => Err(crate::Error::NoRecordFound),
    }
}

pub async fn create(
    ci_id: Uuid,
    createset: CIChangeCreateset,
    pool: &DbPool,
) -> Result<CIChange, crate::Error> {
    createset.validate()?;
    let mut tx = pool.begin().await?;
    check_valid_ci(ci_id, &mut *tx).await?;
    let created_change = sqlx::query_as!(
        CIChange,
        "
        INSERT INTO ci_changes (ci_id, implementation_timedate, documentation)
        VALUES ($1, $2, $3)
        RETURNING id, ci_id, implementation_timedate, documentation",
        ci_id,
        createset.implementation_timedate,
        createset.documentation,
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(crate::Error::DbError)?;

    Ok(created_change)
}

pub async fn update(
    id: Uuid,
    ci_id: Uuid,
    updateset: CIChangeUpdateset,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<CIChange, crate::Error> {
    updateset.validate()?;

    match sqlx::query_as!(
        CIChange,
        "
        UPDATE ci_changes
        SET implementation_timedate = COALESCE($1, implementation_timedate),
            documentation = COALESCE($2, documentation)
        WHERE id = $3
        AND ci_id = $4
        RETURNING id, ci_id, implementation_timedate, documentation",
        updateset.implementation_timedate.unwrap_or(None),
        updateset.documentation.unwrap_or(None),
        id,
        ci_id,
    )
    .fetch_optional(executor)
    .await
    .map_err(crate::Error::DbError)?
    {
        Some(updated_change) => Ok(updated_change),
        None => Err(crate::Error::NoRecordFound),
    }
}

pub async fn delete(
    id: Uuid,
    ci_id: Uuid,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<(), crate::Error> {
    match sqlx::query!(
        "
        DELETE FROM ci_changes
        WHERE id = $1
        AND ci_id = $2
        RETURNING id",
        id,
        ci_id,
    )
    .fetch_optional(executor)
    .await
    .map_err(crate::Error::DbError)?
    {
        Some(_) => Ok(()),
        None => Err(crate::Error::NoRecordFound),
    }
}
