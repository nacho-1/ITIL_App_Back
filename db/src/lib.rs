//! The itil-back-db crate contains all code related to database access: entities, migrations, functions for validating and reading and writing data.

use anyhow::{Context, Result};
use itil_back_config::DatabaseConfig;
use sqlx::{postgres::PgPoolOptions, Postgres, Transaction};
use thiserror::Error;

pub use sqlx::postgres::PgPool as DbPool;

/// Entity definitions and related functions.
pub mod entities;
/// Helper functions for entities.
pub mod entity_helpers;

/// Starts a new database transaction.
///
/// Example:
/// ```
/// let tx = transaction(&app_state.db_pool).await?;
/// tasks::create(task_data, &mut *tx)?;
/// users::create(user_data, &mut *tx)?;
///
/// match tx.commit().await {
///     Ok(_) => Ok((StatusCode::CREATED, Json(results))),
///     Err(e) => Err((internal_error(e), "".into())),
/// }
/// ```
///
/// Transactions are rolled back automatically when they are dropped without having been committed.
pub async fn transaction(
    db_pool: &DbPool,
) -> Result<Transaction<'static, Postgres>, anyhow::Error> {
    let tx = db_pool
        .begin()
        .await
        .context("Failed to begin transaction")?;

    Ok(tx)
}

/// Errors that can occur as a result of a data layer operation.
#[derive(Error, Debug)]
pub enum Error {
    /// General database error, e.g. communicating with the database failed.
    #[error("database query failed")]
    DbError(#[from] sqlx::Error),
    /// No record was found, e.g. when loading a record by ID. This variant is different from
    /// `Error::DbError(sqlx::Error::RowNotFound)` in that the latter indicates a bug, and
    /// `Error::NoRecordFound` does not. It merely originates from [sqlx::Executor::fetch_optional]
    /// returning `None`.
    #[error("no record found")]
    NoRecordFound,
    #[error("validation failed")]
    /// An invalid changeset was passed to a writing operation such as creating or updating a record.
    ValidationError(#[from] validator::ValidationErrors),
    /// Request violated database constraints.
    #[error("constraint violation")]
    ConstraintError,
}

/// Creates a connection pool to the database specified in the passed [`itil-back-config::DatabaseConfig`]
pub async fn connect_pool(config: DatabaseConfig) -> Result<DbPool, anyhow::Error> {
    let pool = PgPoolOptions::new()
        .connect(config.url.as_str())
        .await
        .context("Failed to connect to database")?;

    Ok(pool)
}

/// Functionality for working with data that is only relevant in tests but not as part of the normal application flow.
#[cfg(feature = "test-helpers")]
pub mod test_helpers;
