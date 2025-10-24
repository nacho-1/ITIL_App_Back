use validator::ValidationError;

pub fn validate_not_null<T>(field: &Option<Option<T>>) -> Result<(), ValidationError> {
    if let Some(None) = field {
        return Err(ValidationError::new("Field cannot be null"));
    }

    Ok(())
}
