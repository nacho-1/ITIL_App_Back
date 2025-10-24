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

/// Configuration Item in the database.
#[derive(Debug, Serialize, ToSchema)]
#[cfg_attr(any(feature = "test-helpers", test), derive(Deserialize, PartialEq))]
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

/// Payload for creating a Configuration Item.
#[derive(Clone, Deserialize, ToSchema, Validate)]
#[cfg_attr(any(feature = "test-helpers", test), derive(Serialize))]
pub struct ConfigItemCreateset {
    #[validate(length(min = 1, max = 255))]
    #[schema(example = "IBM 5100")]
    pub name: String,
    pub status: Option<CIStatus>,
    pub created_at: Option<DateTime<Utc>>,
    #[validate(length(max = 1024))]
    #[schema(example = "Workstation")]
    pub r#type: Option<String>,
    #[validate(length(max = 1024))]
    #[schema(example = "IT Department")]
    pub owner: Option<String>,
    #[validate(length(max = 1024))]
    #[schema(example = "Retro portable computer.")]
    pub description: String,
}

/// Payload for updating a Configuration Item.
#[derive(Clone, Deserialize, ToSchema, Validate)]
#[validate(schema(function = "validate_required_fields"))]
#[cfg_attr(any(feature = "test-helpers", test), derive(Serialize))]
pub struct ConfigItemUpdateset {
    #[schema(example = "IBM 5100")]
    #[validate(length(min = 1, max = 255))]
    #[serde(default, with = "::serde_with::rust::double_option")]
    #[cfg_attr(
        any(feature = "test-helpers", test),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub name: Option<Option<String>>,
    #[serde(default, with = "::serde_with::rust::double_option")]
    #[cfg_attr(
        any(feature = "test-helpers", test),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub status: Option<Option<CIStatus>>,
    #[serde(default, with = "::serde_with::rust::double_option")]
    #[cfg_attr(
        any(feature = "test-helpers", test),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub created_at: Option<Option<DateTime<Utc>>>,
    #[schema(example = "Workstation")]
    #[validate(length(max = 1024))]
    #[serde(default, with = "::serde_with::rust::double_option")]
    #[cfg_attr(
        any(feature = "test-helpers", test),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub r#type: Option<Option<String>>,
    #[schema(example = "IT Department")]
    #[validate(length(max = 1024))]
    #[serde(default, with = "::serde_with::rust::double_option")]
    #[cfg_attr(
        any(feature = "test-helpers", test),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub owner: Option<Option<String>>,
    #[schema(example = "Retro portable computer.")]
    #[validate(length(max = 1024))]
    #[serde(default, with = "::serde_with::rust::double_option")]
    #[cfg_attr(
        any(feature = "test-helpers", test),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub description: Option<Option<String>>,
}

/// Validate that required fields of [ConfigItemUpdateset] aren't explicitly null.
fn validate_required_fields(updateset: &ConfigItemUpdateset) -> Result<(), ValidationError> {
    entity_helpers::validate_not_null(&updateset.name)?;
    entity_helpers::validate_not_null(&updateset.status)?;
    entity_helpers::validate_not_null(&updateset.created_at)?;
    entity_helpers::validate_not_null(&updateset.description)?;

    Ok(())
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, ToSchema, Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "cistatus", rename_all = "lowercase")]
#[schema(example = "active")]
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
    configitem: ConfigItemCreateset,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<ConfigItem, crate::Error> {
    configitem.validate()?;

    let created_ci = sqlx::query_as!(
        ConfigItem,
        "
        INSERT INTO configitems (name, status, created_at, type, owner, description)
        VALUES ($1, $2, COALESCE($3, now()), $4, $5, $6)
        RETURNING id, name, status as \"status: CIStatus\", created_at, type, owner, description",
        configitem.name,
        configitem.status.unwrap_or(CIStatus::Inactive) as CIStatus,
        configitem.created_at,
        configitem.r#type,
        configitem.owner,
        configitem.description,
    )
    .fetch_one(executor)
    .await
    .map_err(crate::Error::DbError)?;

    Ok(created_ci)
}

pub async fn update(
    id: Uuid,
    configitem: ConfigItemUpdateset,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<ConfigItem, crate::Error> {
    configitem.validate()?;

    match sqlx::query_as!(
        ConfigItem,
        "
        UPDATE configitems
        SET name = COALESCE($1, name), status = COALESCE($2, status), created_at = COALESCE($3, created_at),
            type = CASE
                WHEN $4 then type
                ELSE $5
            END,
            owner = CASE
                WHEN $6 then owner
                ELSE $7
            END,
            description = COALESCE($8, description)
        WHERE id = $9
        RETURNING id, name, status as \"status: CIStatus\", created_at, type, owner, description",
        configitem.name.unwrap_or(None),
        configitem.status.unwrap_or(None) as Option<CIStatus>,
        configitem.created_at.unwrap_or(None),
        configitem.r#type.is_none(),
        configitem.r#type.unwrap_or(None),
        configitem.owner.is_none(),
        configitem.owner.unwrap_or(None),
        configitem.description.unwrap_or(None),
        id,
    )
    .fetch_optional(executor)
    .await
    .map_err(crate::Error::DbError)?
    {
        Some(updated_ci) => Ok(updated_ci),
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
    fn test_serialize_complete_createset() {
        let set = ConfigItemCreateset {
            name: String::from("Testing Configuration Item"),
            status: Some(CIStatus::Active),
            created_at: Some("2023-09-15T12:34:56Z".parse().unwrap()),
            r#type: Some(String::from("Testing")),
            owner: Some(String::from("Testing Area")),
            description: String::from("This is a testing configuration item."),
        };

        let json = serde_json::to_string(&set).expect("Failed to serialize");

        assert!(json.contains("\"name\":\"Testing Configuration Item\""));
        assert!(json.contains("\"status\":\"active\""));
        assert!(json.contains("\"created_at\":\"2023-09-15T12:34:56Z\""));
        assert!(json.contains("\"type\":\"Testing\""));
        assert!(json.contains("\"owner\":\"Testing Area\""));
        assert!(json.contains("\"description\":\"This is a testing configuration item.\""));
    }

    #[test]
    fn test_serialize_incomplete_createset() {
        let set = ConfigItemCreateset {
            name: String::from("Testing Configuration Item"),
            status: None,
            created_at: None,
            r#type: None,
            owner: None,
            description: String::from("This is a testing configuration item."),
        };

        let json = serde_json::to_string(&set).expect("Failed to serialize");

        assert!(json.contains("\"name\":\"Testing Configuration Item\""));
        assert!(json.contains("\"status\":null"));
        assert!(json.contains("\"created_at\":null"));
        assert!(json.contains("\"type\":null"));
        assert!(json.contains("\"owner\":null"));
        assert!(json.contains("\"description\":\"This is a testing configuration item.\""));
    }

    #[test]
    fn test_deserialize_complete_createset() {
        let json = r#"
        {
            "name": "T1",
            "status": "maintenance",
            "created_at": "2023-09-15T12:34:56Z",
            "type": "Testing",
            "owner": "Test Area",
            "description": "My desc."
        }"#;

        let set: ConfigItemCreateset = serde_json::from_str(json).unwrap();

        assert_eq!(set.name, String::from("T1"));
        assert_eq!(set.status, Some(CIStatus::Maintenance));
        assert_eq!(
            set.created_at,
            Some("2023-09-15T12:34:56Z".parse().unwrap())
        );
        assert_eq!(set.r#type, Some(String::from("Testing")));
        assert_eq!(set.owner, Some(String::from("Test Area")));
        assert_eq!(set.description, String::from("My desc."));
    }

    #[test]
    fn test_deserialize_incomplete_createset() {
        let json = r#"
        {
            "name": "T1",
            "status": null,
            "created_at": null,
            "type": null,
            "description": ""
        }"#;

        let changeset: ConfigItemCreateset = serde_json::from_str(json).unwrap();

        assert_eq!(changeset.name, String::from("T1"));
        assert_eq!(changeset.status, None);
        assert_eq!(changeset.created_at, None);
        assert_eq!(changeset.r#type, None);
        assert_eq!(changeset.owner, None);
        assert_eq!(changeset.description, String::from(""));
    }

    #[test]
    fn test_serialize_complete_updateset() {
        let set = ConfigItemUpdateset {
            name: Some(Some(String::from("Testing Configuration Item"))),
            status: Some(Some(CIStatus::Active)),
            created_at: Some(Some("2023-09-15T12:34:56Z".parse().unwrap())),
            r#type: Some(Some(String::from("Testing"))),
            owner: Some(Some(String::from("Testing Area"))),
            description: Some(Some(String::from("This is a testing configuration item."))),
        };

        let json = serde_json::to_string(&set).expect("Failed to serialize");

        assert!(json.contains("\"name\":\"Testing Configuration Item\""));
        assert!(json.contains("\"status\":\"active\""));
        assert!(json.contains("\"created_at\":\"2023-09-15T12:34:56Z\""));
        assert!(json.contains("\"type\":\"Testing\""));
        assert!(json.contains("\"owner\":\"Testing Area\""));
        assert!(json.contains("\"description\":\"This is a testing configuration item.\""));
    }

    #[test]
    fn test_serialize_complete_updateset_with_nulls() {
        let set = ConfigItemUpdateset {
            name: Some(None),
            status: Some(None),
            created_at: Some(None),
            r#type: Some(None),
            owner: Some(None),
            description: Some(None),
        };

        let json = serde_json::to_string(&set).expect("Failed to serialize");

        assert!(json.contains("\"name\":null"));
        assert!(json.contains("\"status\":null"));
        assert!(json.contains("\"created_at\":null"));
        assert!(json.contains("\"type\":null"));
        assert!(json.contains("\"owner\":null"));
        assert!(json.contains("\"description\":null"));
    }

    #[test]
    fn test_serialize_empty_updateset() {
        let set = ConfigItemUpdateset {
            name: None,
            status: None,
            created_at: None,
            r#type: None,
            owner: None,
            description: None,
        };

        let json = serde_json::to_string(&set).expect("Failed to serialize");

        assert_eq!(json, "{}");
    }

    #[test]
    fn test_deserialize_complete_updateset() {
        let json = r#"
        {
            "name": "T1",
            "status": "maintenance",
            "created_at": "2023-09-15T12:34:56Z",
            "type": "Testing",
            "owner": "Test Area",
            "description": "My desc."
        }"#;

        let set: ConfigItemUpdateset = serde_json::from_str(json).unwrap();

        assert_eq!(set.name, Some(Some(String::from("T1"))));
        assert_eq!(set.status, Some(Some(CIStatus::Maintenance)));
        assert_eq!(
            set.created_at,
            Some(Some("2023-09-15T12:34:56Z".parse().unwrap()))
        );
        assert_eq!(set.r#type, Some(Some(String::from("Testing"))));
        assert_eq!(set.owner, Some(Some(String::from("Test Area"))));
        assert_eq!(set.description, Some(Some(String::from("My desc."))));
    }

    #[test]
    fn test_deserialize_complete_updateset_with_nulls() {
        let json = r#"
        {
            "name": null,
            "status": null,
            "created_at": null,
            "type": null,
            "owner": null,
            "description": null
        }"#;

        let set: ConfigItemUpdateset = serde_json::from_str(json).unwrap();

        assert_eq!(set.name, Some(None));
        assert_eq!(set.status, Some(None));
        assert_eq!(set.created_at, Some(None));
        assert_eq!(set.r#type, Some(None));
        assert_eq!(set.owner, Some(None));
        assert_eq!(set.description, Some(None));
    }

    #[test]
    fn test_deserialize_empty_updateset() {
        let json = r#"{}"#;

        let set: ConfigItemUpdateset = serde_json::from_str(json).unwrap();

        assert_eq!(set.name, None);
        assert_eq!(set.status, None);
        assert_eq!(set.created_at, None);
        assert_eq!(set.r#type, None);
        assert_eq!(set.owner, None);
        assert_eq!(set.description, None);
    }

    #[test]
    fn test_complete_ci_equals() {
        let uuid: Uuid = "550e8400-e29b-41d4-a716-446655440000".parse().unwrap();
        let datetime = Utc::now();

        let ci_1 = ConfigItem {
            name: String::from("x"),
            status: CIStatus::Maintenance,
            id: uuid.clone(),
            created_at: datetime.clone(),
            r#type: Some(String::from("x")),
            owner: Some(String::from("x")),
            description: String::from("x"),
        };
        let ci_2 = ConfigItem {
            name: String::from("x"),
            status: CIStatus::Maintenance,
            id: uuid.clone(),
            created_at: datetime.clone(),
            r#type: Some(String::from("x")),
            owner: Some(String::from("x")),
            description: String::from("x"),
        };

        assert_eq!(ci_1, ci_2);
    }
}
