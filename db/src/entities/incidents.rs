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

/// Module for handling relations between Configuration Items and Incidents.
pub mod ci_relations;

#[derive(Debug, Deserialize)]
pub struct Incident {
    pub id: Uuid,
    pub title: String,
    pub status: IncidentStatus,
    pub created_at: DateTime<Utc>,
    pub impact: IncidentImpact,
    pub urgency: IncidentUrgency,
    pub owner: Option<String>,
    pub description: String,
}

impl Incident {
    pub fn priority(&self) -> IncidentPrio {
        IncidentPrio::from(&self.impact, &self.urgency)
    }
}

impl Serialize for Incident {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Incident", 9)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("title", &self.title)?;
        state.serialize_field("status", &self.status)?;
        state.serialize_field("created_at", &self.created_at)?;
        state.serialize_field("impact", &self.impact)?;
        state.serialize_field("urgency", &self.urgency)?;
        state.serialize_field("priority", &self.priority())?;
        state.serialize_field("owner", &self.owner)?;
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
            pub impact: IncidentImpact,
            pub urgency: IncidentUrgency,
            pub priority: IncidentPrio,
            #[schema(example = "Sales Department")]
            pub owner: Option<String>,
            #[schema(example = "Proxy server not working. Stopped this morning.")]
            pub description: String,
        }

        IncidentSchema::schema()
    }
}

#[derive(Deserialize, Validate, Clone, ToSchema)]
#[cfg_attr(feature = "test-helpers", derive(Serialize))]
pub struct IncidentChangeset {
    #[validate(length(min = 1, max = 255))]
    #[schema(example = "Proxy Not Working")]
    pub title: String,
    pub status: IncidentStatus,
    pub created_at: Option<DateTime<Utc>>,
    pub impact: IncidentImpact,
    pub urgency: IncidentUrgency,
    #[validate(length(min = 1, max = 63))]
    #[schema(example = "Sales Department")]
    pub owner: Option<String>,
    #[validate(length(max = 255))]
    #[schema(example = "Proxy server not working. Stopped this morning.")]
    pub description: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, ToSchema)]
#[schema(example = "open")]
#[sqlx(type_name = "incident_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
#[cfg_attr(any(feature = "test-helpers", test), derive(PartialEq))]
pub enum IncidentStatus {
    Open,
    InProgress,
    Closed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, ToSchema)]
#[schema(example = "high")]
#[sqlx(type_name = "incident_impact", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, ToSchema)]
#[schema(example = "high")]
#[sqlx(type_name = "incident_urgency", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, ToSchema)]
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
        SELECT id, title, status as \"status: IncidentStatus\", created_at,
            impact as \"impact: IncidentImpact\", urgency as \"urgency: IncidentUrgency\",
            owner, description
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
        SELECT id, title, status as \"status: IncidentStatus\", created_at,
            impact as \"impact: IncidentImpact\", urgency as \"urgency: IncidentUrgency\",
            owner, description
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
    incident: IncidentChangeset,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<Incident, crate::Error> {
    incident.validate()?;

    let record = sqlx::query!(
        "
        INSERT INTO incidents (title, status, created_at, impact, urgency, owner, description) 
        VALUES ($1, $2, COALESCE($3, now()), $4, $5, $6, $7) 
        RETURNING id, created_at",
        incident.title,
        incident.status as IncidentStatus,
        incident.created_at,
        incident.impact as IncidentImpact,
        incident.urgency as IncidentUrgency,
        incident.owner,
        incident.description,
    )
    .fetch_one(executor)
    .await
    .map_err(crate::Error::DbError)?;

    Ok(Incident {
        id: record.id,
        title: incident.title,
        status: incident.status,
        created_at: record.created_at,
        impact: incident.impact,
        urgency: incident.urgency,
        owner: incident.owner,
        description: incident.description,
    })
}

pub async fn update(
    id: Uuid,
    incident: IncidentChangeset,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<Incident, crate::Error> {
    incident.validate()?;

    match sqlx::query!(
        "
        UPDATE incidents
        SET title = $1, status = $2, created_at = COALESCE($3, created_at),
            impact = $4, urgency = $5, owner = $6, description = $7 
        WHERE id = $8
        RETURNING id, created_at",
        incident.title,
        incident.status as IncidentStatus,
        incident.created_at,
        incident.impact as IncidentImpact,
        incident.urgency as IncidentUrgency,
        incident.owner,
        incident.description,
        id,
    )
    .fetch_optional(executor)
    .await
    .map_err(crate::Error::DbError)?
    {
        Some(record) => Ok(Incident {
            id: record.id,
            title: incident.title,
            status: incident.status,
            created_at: record.created_at,
            impact: incident.impact,
            urgency: incident.urgency,
            owner: incident.owner,
            description: incident.description,
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
            impact: IncidentImpact::Low,
            urgency: IncidentUrgency::Low,
            owner: Some(String::from("Me")),
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
