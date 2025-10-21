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
    #[schema(example = "IBM 5100")]
    pub name: String,
    pub status: CIStatus,
    pub created_at: DateTime<Utc>,
    #[schema(example = "Workstation")]
    pub r#type: Option<String>,
    #[schema(example = "IT Department")]
    pub owner: Option<String>,
    #[schema(example = "Retro portable computer.")]
    pub description: String,
}

#[derive(Deserialize, Validate, Clone, ToSchema)]
#[cfg_attr(any(feature = "test-helpers", test), derive(Serialize))]
pub struct ConfigItemChangeset {
    #[validate(length(min = 1, max = 255))]
    #[schema(example = "IBM 5100")]
    pub name: String,
    pub status: CIStatus,
    pub created_at: Option<DateTime<Utc>>,
    #[validate(length(min = 1, max = 31))]
    #[schema(example = "Workstation")]
    pub r#type: Option<String>,
    #[validate(length(min = 1, max = 63))]
    #[schema(example = "IT Department")]
    pub owner: Option<String>,
    #[validate(length(max = 255))]
    #[schema(example = "Retro portable computer.")]
    pub description: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, ToSchema)]
#[schema(example = "active")]
#[sqlx(type_name = "cistatus", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
#[cfg_attr(any(feature = "test-helpers", test), derive(PartialEq))]
pub enum CIStatus {
    Active,
    Inactive,
    Maintenance,
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
        VALUES ($1, $2, COALESCE($3, now()), $4, $5, $6) 
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

#[cfg(test)]
mod config_item_test {
    use super::*;

    #[test]
    fn test_serialize_changeset() {
        let changeset = ConfigItemChangeset {
            name: String::from("Testing Configuration Item"),
            status: CIStatus::Active,
            created_at: Some("2023-09-15T12:34:56Z".parse().unwrap()),
            r#type: Some(String::from("Testing")),
            owner: Some(String::from("Testing Area")),
            description: String::from("This is a testing configuration item."),
        };

        let json = serde_json::to_string(&changeset).expect("Failed to serialize");

        assert!(json.contains("\"name\":\"Testing Configuration Item\""));
        assert!(json.contains("\"status\":\"active\""));
        assert!(json.contains("\"created_at\":\"2023-09-15T12:34:56Z\""));
        assert!(json.contains("\"status\":\"active\""));
        assert!(json.contains("\"description\":\"This is a testing configuration item.\""));
    }

    #[test]
    fn test_deserialize_complete_changeset() {
        let json = r#"
        {
            "name": "T1",
            "status": "maintenance",
            "created_at": "2023-09-15T12:34:56Z",
            "type": "Testing",
            "owner": "Test Area",
            "description": "My desc."
        }"#;

        let changeset: ConfigItemChangeset = serde_json::from_str(json).unwrap();

        assert_eq!(changeset.name, String::from("T1"));
        assert_eq!(changeset.status, CIStatus::Maintenance);
        assert_eq!(
            changeset.created_at,
            Some("2023-09-15T12:34:56Z".parse().unwrap())
        );
        assert_eq!(changeset.r#type, Some(String::from("Testing")));
        assert_eq!(changeset.owner, Some(String::from("Test Area")));
        assert_eq!(changeset.description, String::from("My desc."));
    }

    #[test]
    fn test_deserialize_incomplete_changeset() {
        let json = r#"
        {
            "name": "T1",
            "status": "maintenance",
            "created_at": null,
            "type": null,
            "description": ""
        }"#;

        let changeset: ConfigItemChangeset = serde_json::from_str(json).unwrap();

        assert_eq!(changeset.name, String::from("T1"));
        assert_eq!(changeset.status, CIStatus::Maintenance);
        assert_eq!(changeset.created_at, None);
        assert_eq!(changeset.r#type, None);
        assert_eq!(changeset.owner, None);
        assert_eq!(changeset.description, String::from(""));
    }
}
