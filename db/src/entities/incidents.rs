use crate::entity_helpers;
use serde::ser::SerializeStruct;
use serde::Deserialize;
use serde::Serialize;
use sqlx::types::chrono::DateTime;
use sqlx::types::chrono::Utc;
use sqlx::Postgres;
use sqlx::Type;
use utoipa::PartialSchema;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;
use validator::ValidationError;

/// Module for handling relations between Configuration Items and Incidents.
pub mod ci_relations;

#[derive(Debug)]
#[cfg_attr(any(feature = "test-helpers", test), derive(Deserialize, PartialEq))]
pub struct Incident {
    pub id: Uuid,
    pub title: String,
    pub status: IncidentStatus,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub impact: IncidentImpact,
    pub urgency: IncidentUrgency,
    pub owner: Option<String>,
    pub asignee: Option<String>,
    pub description: String,
}

impl Incident {
    pub fn priority(&self) -> IncidentPrio {
        IncidentPrio::from(&self.impact, &self.urgency)
    }
}

// Manually implement serialize cause priority is not an actual field.
impl Serialize for Incident {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Incident", 10)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("title", &self.title)?;
        state.serialize_field("status", &self.status)?;
        state.serialize_field("created_at", &self.created_at)?;
        state.serialize_field("resolved_at", &self.resolved_at)?;
        state.serialize_field("impact", &self.impact)?;
        state.serialize_field("urgency", &self.urgency)?;
        state.serialize_field("priority", &self.priority())?;
        state.serialize_field("owner", &self.owner)?;
        state.serialize_field("asignee", &self.asignee)?;
        state.serialize_field("description", &self.description)?;
        state.end()
    }
}

impl ToSchema for Incident {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("Incident")
    }
}

impl PartialSchema for Incident {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        /// Helper struct for generating the schemas for [Incident].
        ///
        /// Because the schema should include the priority, which isn't an actual field,
        /// I do this workaround.
        #[derive(ToSchema)]
        #[allow(dead_code)]
        struct IncidentSchema {
            pub id: Uuid,
            #[schema(example = "Proxy Not Working")]
            pub title: String,
            pub status: IncidentStatus,
            pub created_at: DateTime<Utc>,
            pub resolved_at: Option<DateTime<Utc>>,
            pub impact: IncidentImpact,
            pub urgency: IncidentUrgency,
            pub priority: IncidentPrio,
            #[schema(example = "Sales Department")]
            pub owner: Option<String>,
            #[schema(example = "Employee 1837")]
            pub asignee: Option<String>,
            #[schema(example = "Proxy server not working. Stopped this morning.")]
            pub description: String,
        }

        IncidentSchema::schema()
    }
}

/// Payload for creating an Incident.
#[derive(Clone, Deserialize, ToSchema, Validate)]
#[cfg_attr(feature = "test-helpers", derive(Serialize))]
pub struct IncidentCreateset {
    #[schema(example = "Proxy Not Working")]
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    #[schema(example = "open")]
    pub status: Option<IncidentStatus>,
    pub created_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub impact: IncidentImpact,
    pub urgency: IncidentUrgency,
    #[schema(example = "Sales Department")]
    #[validate(length(max = 1024))]
    pub owner: Option<String>,
    #[schema(example = "Employee 1837")]
    #[validate(length(max = 1024))]
    pub asignee: Option<String>,
    #[schema(example = "Proxy server not working. Stopped this morning.")]
    #[validate(length(max = 1024))]
    pub description: String,
}

/// Payload for updating an Incident.
#[derive(Clone, Deserialize, ToSchema, Validate)]
#[validate(schema(function = "validate_required_fields"))]
#[cfg_attr(feature = "test-helpers", derive(Serialize))]
pub struct IncidentUpdateset {
    #[schema(example = "Proxy Not Working")]
    #[validate(length(min = 1, max = 255))]
    #[serde(default, with = "::serde_with::rust::double_option")]
    #[cfg_attr(
        any(feature = "test-helpers", test),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub title: Option<Option<String>>,
    #[schema(example = "open")]
    #[serde(default, with = "::serde_with::rust::double_option")]
    #[cfg_attr(
        any(feature = "test-helpers", test),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub status: Option<Option<IncidentStatus>>,
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
    pub resolved_at: Option<Option<DateTime<Utc>>>,
    #[serde(default, with = "::serde_with::rust::double_option")]
    #[cfg_attr(
        any(feature = "test-helpers", test),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub impact: Option<Option<IncidentImpact>>,
    #[serde(default, with = "::serde_with::rust::double_option")]
    #[cfg_attr(
        any(feature = "test-helpers", test),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub urgency: Option<Option<IncidentUrgency>>,
    #[schema(example = "Sales Department")]
    #[validate(length(max = 1024))]
    #[serde(default, with = "::serde_with::rust::double_option")]
    #[cfg_attr(
        any(feature = "test-helpers", test),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub owner: Option<Option<String>>,
    #[schema(example = "Employee 1837")]
    #[validate(length(max = 1024))]
    #[serde(default, with = "::serde_with::rust::double_option")]
    #[cfg_attr(
        any(feature = "test-helpers", test),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub asignee: Option<Option<String>>,
    #[schema(example = "Proxy server not working. Stopped this morning.")]
    #[validate(length(max = 1024))]
    #[serde(default, with = "::serde_with::rust::double_option")]
    #[cfg_attr(
        any(feature = "test-helpers", test),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub description: Option<Option<String>>,
}

/// Validate that required fields of [IncidentUpdateset] aren't explicitly null.
fn validate_required_fields(updateset: &IncidentUpdateset) -> Result<(), ValidationError> {
    entity_helpers::validate_not_null(&updateset.title)?;
    entity_helpers::validate_not_null(&updateset.status)?;
    entity_helpers::validate_not_null(&updateset.created_at)?;
    entity_helpers::validate_not_null(&updateset.impact)?;
    entity_helpers::validate_not_null(&updateset.urgency)?;
    entity_helpers::validate_not_null(&updateset.description)?;

    Ok(())
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, ToSchema, Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "incident_status", rename_all = "lowercase")]
#[schema(example = "open")]
#[cfg_attr(any(feature = "test-helpers", test), derive(PartialEq))]
pub enum IncidentStatus {
    Open,
    InProgress,
    Closed,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, ToSchema, Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "incident_impact", rename_all = "lowercase")]
#[schema(example = "high")]
#[cfg_attr(any(feature = "test-helpers", test), derive(PartialEq))]
pub enum IncidentImpact {
    High,
    Medium,
    Low,
}

impl IncidentImpact {
    pub fn weight(&self) -> u8 {
        match self {
            Self::High => 3,
            Self::Medium => 2,
            Self::Low => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, ToSchema, Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "incident_urgency", rename_all = "lowercase")]
#[schema(example = "high")]
#[cfg_attr(any(feature = "test-helpers", test), derive(PartialEq))]
pub enum IncidentUrgency {
    High,
    Medium,
    Low,
}

impl IncidentUrgency {
    pub fn weight(&self) -> u8 {
        match self {
            Self::High => 3,
            Self::Medium => 2,
            Self::Low => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, ToSchema, Type)]
#[schema(example = "critical")]
#[serde(rename_all = "lowercase")]
#[cfg_attr(any(feature = "test-helpers", test), derive(PartialEq))]
pub enum IncidentPrio {
    Critical,
    High,
    Moderate,
    Low,
}

impl IncidentPrio {
    pub fn from(impact: &IncidentImpact, urgency: &IncidentUrgency) -> Self {
        let coef = impact.weight() * urgency.weight();
        match coef {
            1 | 2 => IncidentPrio::Low,
            3 | 4 => IncidentPrio::Moderate,
            6 => IncidentPrio::High,
            9 => IncidentPrio::Critical,
            _ => panic!(
                "Bad values.\n\timpact: {}\n\turgency: {}",
                impact.weight(),
                urgency.weight()
            ),
        }
    }
}

pub async fn load_all(
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<Vec<Incident>, crate::Error> {
    let incidents = sqlx::query_as!(
        Incident,
        "
        SELECT id, title, status as \"status: IncidentStatus\", created_at, resolved_at,
            impact as \"impact: IncidentImpact\", urgency as \"urgency: IncidentUrgency\",
            owner, asignee, description
        FROM incidents"
    )
    .fetch_all(executor)
    .await?;

    Ok(incidents)
}

pub async fn load(
    id: Uuid,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<Incident, crate::Error> {
    match sqlx::query_as!(
        Incident,
        "
        SELECT id, title, status as \"status: IncidentStatus\", created_at, resolved_at,
            impact as \"impact: IncidentImpact\", urgency as \"urgency: IncidentUrgency\",
            owner, asignee, description
        FROM incidents
        WHERE id = $1",
        id
    )
    .fetch_optional(executor)
    .await
    .map_err(crate::Error::DbError)?
    {
        Some(incident) => Ok(incident),
        None => Err(crate::Error::NoRecordFound),
    }
}

pub async fn create(
    createset: IncidentCreateset,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<Incident, crate::Error> {
    createset.validate()?;

    let created_incident = sqlx::query_as!(
        Incident,
        "
        INSERT INTO incidents (title, status, created_at, resolved_at, impact, urgency,
            owner, asignee, description)
        VALUES ($1, $2, COALESCE($3, now()), $4, $5, $6, $7, $8, $9)
        RETURNING id, title, status as \"status: IncidentStatus\", created_at, resolved_at,
            impact as \"impact: IncidentImpact\", urgency as \"urgency: IncidentUrgency\",
            owner, asignee, description",
        createset.title,
        createset.status.unwrap_or(IncidentStatus::Open) as IncidentStatus,
        createset.created_at,
        createset.resolved_at,
        createset.impact as IncidentImpact,
        createset.urgency as IncidentUrgency,
        createset.owner,
        createset.asignee,
        createset.description,
    )
    .fetch_one(executor)
    .await
    .map_err(crate::Error::DbError)?;

    Ok(created_incident)
}

pub async fn update(
    id: Uuid,
    updateset: IncidentUpdateset,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<Incident, crate::Error> {
    updateset.validate()?;

    match sqlx::query_as!(
        Incident,
        "
        UPDATE incidents
        SET title = COALESCE($1, title), status = COALESCE($2, status), created_at = COALESCE($3, created_at),
            resolved_at = CASE
                WHEN $4 THEN resolved_at
                ELSE $5
            END,
            impact = COALESCE($6, impact), urgency = COALESCE($7, urgency),
            owner = CASE
                WHEN $8 THEN owner
                ELSE $9
            END,
            asignee = CASE
                WHEN $10 THEN asignee
                ELSE $11
            END,
            description = COALESCE($12, description)
        WHERE id = $13
        RETURNING id, title, status as \"status: IncidentStatus\", created_at, resolved_at,
            impact as \"impact: IncidentImpact\", urgency as \"urgency: IncidentUrgency\",
            owner, asignee, description",
        updateset.title.unwrap_or(None),
        updateset.status.unwrap_or(None) as Option<IncidentStatus>,
        updateset.created_at.unwrap_or(None),
        updateset.resolved_at.is_none(),
        updateset.resolved_at.unwrap_or(None),
        updateset.impact.unwrap_or(None) as Option<IncidentImpact>,
        updateset.urgency.unwrap_or(None) as Option<IncidentUrgency>,
        updateset.owner.is_none(),
        updateset.owner.unwrap_or(None),
        updateset.asignee.is_none(),
        updateset.asignee.unwrap_or(None),
        updateset.description.unwrap_or(None),
        id,
    )
    .fetch_optional(executor)
    .await
    .map_err(crate::Error::DbError)?
    {
        Some(updated_incident) => Ok(updated_incident),
        None => Err(crate::Error::NoRecordFound),
    }
}

pub async fn delete(
    id: Uuid,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<(), crate::Error> {
    match sqlx::query!(
        "
        DELETE FROM incidents
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
mod incidents_tests {
    use super::*;
    use uuid::uuid;

    fn build_testing_incident() -> Incident {
        Incident {
            title: String::from("Test Incident"),
            id: uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8"),
            status: IncidentStatus::Open,
            created_at: Utc::now(),
            resolved_at: None,
            impact: IncidentImpact::Low,
            urgency: IncidentUrgency::Low,
            owner: Some(String::from("Me")),
            asignee: Some(String::from("Someone")),
            description: String::from(""),
        }
    }

    #[test]
    fn test_low_priority() {
        let mut incidents = Vec::new();
        incidents.push(build_testing_incident());
        incidents.push(Incident {
            impact: IncidentImpact::Medium,
            ..build_testing_incident()
        });
        incidents.push(Incident {
            urgency: IncidentUrgency::Medium,
            ..build_testing_incident()
        });

        for incident in incidents {
            assert_eq!(incident.priority(), IncidentPrio::Low);
        }
    }

    #[test]
    fn test_moderate_priority() {
        let mut incidents = Vec::new();
        incidents.push(Incident {
            impact: IncidentImpact::High,
            ..build_testing_incident()
        });
        incidents.push(Incident {
            urgency: IncidentUrgency::High,
            ..build_testing_incident()
        });
        incidents.push(Incident {
            impact: IncidentImpact::Medium,
            urgency: IncidentUrgency::Medium,
            ..build_testing_incident()
        });

        for incident in incidents {
            assert_eq!(incident.priority(), IncidentPrio::Moderate);
        }
    }

    #[test]
    fn test_high_priority() {
        let mut incidents = Vec::new();
        incidents.push(Incident {
            impact: IncidentImpact::High,
            urgency: IncidentUrgency::Medium,
            ..build_testing_incident()
        });
        incidents.push(Incident {
            impact: IncidentImpact::Medium,
            urgency: IncidentUrgency::High,
            ..build_testing_incident()
        });

        for incident in incidents {
            assert_eq!(incident.priority(), IncidentPrio::High);
        }
    }

    #[test]
    fn test_critical_priority() {
        let incident = Incident {
            impact: IncidentImpact::High,
            urgency: IncidentUrgency::High,
            ..build_testing_incident()
        };
        assert_eq!(incident.priority(), IncidentPrio::Critical);
    }
}
