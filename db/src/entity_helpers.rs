use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::{Validate, ValidateLength};

/// Custom field to distinguish between None and explicit null.
#[derive(Debug, Deserialize, Serialize, Clone, ToSchema)]
#[serde(untagged)]
pub enum PatchField<T: Clone> {
    /// Field not included in JSON
    Missing,
    /// Field explicitly set to null
    Null,
    /// Field provided with a value
    Value(T),
}

impl<T: Clone> PatchField<T> {
    pub fn as_option(&self) -> Option<T> {
        match self {
            PatchField::Missing | PatchField::Null => None,
            PatchField::Value(v) => Some(v.clone()),
        }
    }

    pub fn leave_unchanged(&self) -> bool {
        match self {
            PatchField::Missing => true,
            _ => false,
        }
    }
}

impl<T: Validate + Clone> Validate for PatchField<T> {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        match self {
            PatchField::Value(inner) => inner.validate(),
            PatchField::Null | PatchField::Missing => Ok(()),
        }
    }
}

impl<T, U> ValidateLength<U> for PatchField<T>
where
    T: ValidateLength<U> + Clone,
    U: PartialEq + PartialOrd,
{
    fn validate_length(&self, min: Option<U>, max: Option<U>, equal: Option<U>) -> bool {
        match self {
            PatchField::Value(inner) => inner.validate_length(min, max, equal),
            PatchField::Null | PatchField::Missing => true,
        }
    }

    fn length(&self) -> Option<U> {
        match self {
            PatchField::Value(inner) => inner.length(),
            PatchField::Null | PatchField::Missing => None,
        }
    }
}
