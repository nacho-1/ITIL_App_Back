//#[cfg(feature = "test-helpers")]
use serde::Deserialize;
use serde::Serialize;
use sqlx::types::chrono::DateTime;
use sqlx::types::chrono::Utc;
use sqlx::Postgres;
use sqlx::Type;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Serialize, Debug, Deserialize, ToSchema)]
pub struct ConfigItem {
    pub id: Uuid,
    pub name: String,
    pub status: CIStatus,
    pub created_at: DateTime<Utc>,
    pub r#type: Option<String>,
    pub owner: Option<String>,
    pub description: String,
}

#[derive(Deserialize, Validate, Clone, ToSchema)]
#[cfg_attr(feature = "test-helpers", derive(Serialize))]
pub struct ConfigItemChangeset {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub status: CIStatus,
    pub created_at: Option<DateTime<Utc>>,
    #[validate(length(min = 1, max = 31))]
    pub r#type: Option<String>,
    #[validate(length(min = 1, max = 63))]
    pub owner: Option<String>,
    #[validate(length(max = 255))]
    pub description: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, ToSchema)]
#[schema(example = "active")]
#[sqlx(type_name = "cistatus", rename_all = "lowercase")]
pub enum CIStatus {
    Active,
    Inactive,
    InMaintenance,
    Testing,
    Retired,
}

pub async fn load_all(
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<Vec<ConfigItem>, crate::Error> {
    let configitems = sqlx::query_as!(
        ConfigItem,
        "
        SELECT id, name, status as \"status: CIStatus\", created_at, type, owner, description 
        FROM configitems"
    )
    .fetch_all(executor)
    .await?;
    Ok(configitems)
}

pub async fn load(
    id: Uuid,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<ConfigItem, crate::Error> {
    match sqlx::query_as!(
        ConfigItem,
        "
        SELECT id, name, status as \"status: CIStatus\", created_at, type, owner, description 
        FROM configitems 
        WHERE id = $1",
        id
    )
    .fetch_optional(executor)
    .await
    .map_err(crate::Error::DbError)?
    {
        Some(configitem) => Ok(configitem),
        None => Err(crate::Error::NoRecordFound),
    }
}

pub async fn create(
    configitem: ConfigItemChangeset,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<ConfigItem, crate::Error> {
    configitem.validate()?;

    let record = sqlx::query!(
        "
        INSERT INTO configitems (name, status, created_at, type, owner, description) 
        VALUES ($1, $2, $3, $4, $5, $6) 
        RETURNING id, created_at",
        configitem.name,
        configitem.status as CIStatus,
        configitem.created_at,
        configitem.r#type,
        configitem.owner,
        configitem.description,
    )
    .fetch_one(executor)
    .await
    .map_err(crate::Error::DbError)?;

    Ok(ConfigItem {
        id: record.id,
        name: configitem.name,
        status: configitem.status,
        created_at: record.created_at,
        r#type: configitem.r#type,
        owner: configitem.owner,
        description: configitem.description,
    })
}

pub async fn update(
    id: Uuid,
    configitem: ConfigItemChangeset,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<ConfigItem, crate::Error> {
    configitem.validate()?;

    match sqlx::query!(
        "
        UPDATE configitems 
        SET name = $1, status = $2, created_at = COALESCE($3, created_at), type = $4, owner = $5, description = $6 
        WHERE id = $7
        RETURNING id, created_at",
        configitem.name,
        configitem.status as CIStatus,
        configitem.created_at,
        configitem.r#type,
        configitem.owner,
        configitem.description,
        id,
    )
    .fetch_optional(executor)
    .await
    .map_err(crate::Error::DbError)?
    {
        Some(record) => Ok(ConfigItem {
            id: record.id,
            name: configitem.name,
            status: configitem.status,
            created_at: record.created_at,
            r#type: configitem.r#type,
            owner: configitem.owner,
            description: configitem.description,
        }),
        None => Err(crate::Error::NoRecordFound),
    }
}

pub async fn delete(
    id: Uuid,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<(), crate::Error> {
    match sqlx::query!(
        "
        DELETE FROM configitems 
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
