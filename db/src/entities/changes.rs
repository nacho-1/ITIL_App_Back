use crate::entity_helpers;
use serde::Deserialize;
use serde::Serialize;
use sqlx::types::chrono::DateTime;
use sqlx::types::chrono::Utc;
use sqlx::Postgres;
use sqlx::Type;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;
use validator::ValidationError;

/// RFC in the database.
#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(any(feature = "test-helpers", test), derive(Deserialize, PartialEq))]
pub struct RFC {
    pub id: Uuid,
    #[schema(example = "Sales Department OS Update")]
    pub title: String,
    pub status: RFCStatus,
    pub created_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    #[schema(example = "Sales Department.")]
    pub requester: String,
    #[schema(example = "Update sales department workstations to naviOS v25.")]
    pub description: String,
}

/// Payload for creating an RFC.
#[derive(Clone, Deserialize, ToSchema, Validate)]
#[cfg_attr(any(feature = "test-helpers", test), derive(Serialize))]
pub struct RFCCreateset {
    #[schema(example = "Sales Department OS Update")]
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    pub status: Option<RFCStatus>,
    pub created_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    #[schema(example = "Sales Department.")]
    #[validate(length(max = 1024))]
    pub requester: String,
    #[schema(example = "Update sales department workstations to naviOS v25.")]
    #[validate(length(max = 1024))]
    pub description: String,
}

/// Payload for updating an RFC.
#[derive(Clone, Deserialize, ToSchema, Validate)]
#[validate(schema(function = "validate_required_fields"))]
#[cfg_attr(any(feature = "test-helpers", test), derive(Serialize))]
pub struct RFCUpdateset {
    #[schema(example = "Sales Department OS Update")]
    #[validate(length(min = 1, max = 255))]
    #[serde(default, with = "::serde_with::rust::double_option")]
    #[cfg_attr(
        any(feature = "test-helpers", test),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub title: Option<Option<String>>,
    #[serde(default, with = "::serde_with::rust::double_option")]
    #[cfg_attr(
        any(feature = "test-helpers", test),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub status: Option<Option<RFCStatus>>,
    #[serde(default, with = "::serde_with::rust::double_option")]
    #[cfg_attr(
        any(feature = "test-helpers", test),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub created_at: Option<Option<DateTime<Utc>>>,
    #[serde(default, with = "::serde_with::rust::double_option")]
    #[cfg_attr(
        any(feature = "test-helpers", test),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub finished_at: Option<Option<DateTime<Utc>>>,
    #[schema(example = "Sales Department")]
    #[validate(length(max = 1024))]
    #[serde(default, with = "::serde_with::rust::double_option")]
    #[cfg_attr(
        any(feature = "test-helpers", test),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub requester: Option<Option<String>>,
    #[schema(example = "Update sales department workstations to naviOS v25.")]
    #[validate(length(max = 1024))]
    #[serde(default, with = "::serde_with::rust::double_option")]
    #[cfg_attr(
        any(feature = "test-helpers", test),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub description: Option<Option<String>>,
}

/// Validate that required fields of [RFCUpdateset] aren't explicitly null.
fn validate_required_fields(updateset: &RFCUpdateset) -> Result<(), ValidationError> {
    entity_helpers::validate_not_null(&updateset.title)?;
    entity_helpers::validate_not_null(&updateset.status)?;
    entity_helpers::validate_not_null(&updateset.created_at)?;
    entity_helpers::validate_not_null(&updateset.requester)?;
    entity_helpers::validate_not_null(&updateset.description)?;

    Ok(())
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, ToSchema, Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "rfcstatus", rename_all = "lowercase")]
#[schema(example = "active")]
#[cfg_attr(any(feature = "test-helpers", test), derive(PartialEq))]
pub enum RFCStatus {
    Open,
    InProgress,
    Closed,
}

pub async fn load_all(
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<Vec<RFC>, crate::Error> {
    let rfcs = sqlx::query_as!(
        RFC,
        "
        SELECT id, title, status as \"status: RFCStatus\", created_at, finished_at, requester, description
        FROM rfcs"
    )
    .fetch_all(executor)
    .await?;
    Ok(rfcs)
}

pub async fn load(
    id: Uuid,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<RFC, crate::Error> {
    match sqlx::query_as!(
        RFC,
        "
        SELECT id, title, status as \"status: RFCStatus\", created_at, finished_at, requester, description
        FROM rfcs
        WHERE id = $1",
        id
    )
    .fetch_optional(executor)
    .await
    .map_err(crate::Error::DbError)?
    {
        Some(rfc) => Ok(rfc),
        None => Err(crate::Error::NoRecordFound),
    }
}

pub async fn create(
    createset: RFCCreateset,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<RFC, crate::Error> {
    createset.validate()?;

    let created_rfc = sqlx::query_as!(
        RFC,
        "
        INSERT INTO rfcs (title, status, created_at, finished_at, requester, description)
        VALUES ($1, $2, COALESCE($3, now()), $4, $5, $6)
        RETURNING id, title, status as \"status: RFCStatus\", created_at, finished_at, requester, description",
        createset.title,
        createset.status.unwrap_or(RFCStatus::Open) as RFCStatus,
        createset.created_at,
        createset.finished_at,
        createset.requester,
        createset.description,
    )
    .fetch_one(executor)
    .await
    .map_err(crate::Error::DbError)?;

    Ok(created_rfc)
}

pub async fn update(
    id: Uuid,
    updateset: RFCUpdateset,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<RFC, crate::Error> {
    updateset.validate()?;

    match sqlx::query_as!(
        RFC,
        "
        UPDATE rfcs
        SET title = COALESCE($1, title), status = COALESCE($2, status), created_at = COALESCE($3, created_at),
            finished_at = CASE
                WHEN $4 then finished_at
                ELSE $5
            END,
            requester = COALESCE($6, requester), description = COALESCE($7, description)
        WHERE id = $8
        RETURNING id, title, status as \"status: RFCStatus\", created_at, finished_at, requester, description",
        updateset.title.unwrap_or(None),
        updateset.status.unwrap_or(None) as Option<RFCStatus>,
        updateset.created_at.unwrap_or(None),
        updateset.finished_at.is_none(),
        updateset.finished_at.unwrap_or(None),
        updateset.requester.unwrap_or(None),
        updateset.description.unwrap_or(None),
        id,
    )
    .fetch_optional(executor)
    .await
    .map_err(crate::Error::DbError)?
    {
        Some(updated_rfc) => Ok(updated_rfc),
        None => Err(crate::Error::NoRecordFound),
    }
}

pub async fn delete(
    id: Uuid,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<(), crate::Error> {
    match sqlx::query!(
        "
        DELETE FROM rfcs
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
