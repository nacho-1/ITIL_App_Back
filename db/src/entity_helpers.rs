use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::{Validate, ValidateLength};

/// Custom field to distinguish between None and explicit null.
///
/// Some considerations:
/// 1) It's mandatory to use #[serde(default)] when using this enum.
/// Makes it so that when the field is missing, it maps it to [PatchField::Missing].
/// 2) For serialization, use the following macro:
/// #[serde(skip_serializing_if = "PatchField::leave_unchanged")]
/// Makes it so that it properly skips the field when it should.
#[derive(Debug, Clone, ToSchema)]
#[cfg_attr(any(feature = "test-helpers", test), derive(Serialize))]
#[cfg_attr(any(feature = "test-helpers", test), serde(untagged))]
pub enum PatchField<T> {
    /// Field not included in JSON
    Missing,
    /// Field explicitly set to null
    Null,
    /// Field provided with a value
    Value(T),
}

impl<T> PatchField<T> {
    pub fn leave_unchanged(&self) -> bool {
        matches!(self, PatchField::Missing)
    }
}

impl<T> Default for PatchField<T> {
    fn default() -> Self {
        PatchField::Missing
    }
}

impl<T> From<Option<T>> for PatchField<T> {
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(v) => PatchField::Value(v),
            None => PatchField::Null,
        }
    }
}

impl<T> Into<Option<T>> for PatchField<T> {
    fn into(self) -> Option<T> {
        match self {
            PatchField::Missing | PatchField::Null => None,
            PatchField::Value(v) => Some(v),
        }
    }
}

impl<'de, T> Deserialize<'de> for PatchField<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Option::deserialize(deserializer).map(Into::into)
    }
}

impl<T: Validate> Validate for PatchField<T> {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        match self {
            PatchField::Value(inner) => inner.validate(),
            PatchField::Null | PatchField::Missing => Ok(()),
        }
    }
}

impl<T, U> ValidateLength<U> for PatchField<T>
where
    T: ValidateLength<U>,
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
