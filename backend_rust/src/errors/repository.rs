use thiserror::Error;

pub type RepositoryResult<T> = Result<T, RepositoryError>;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("foreign key violation: {0}")]
    ForeignKeyViolation(String),
    #[error("unique violation: {0}")]
    UniqueViolation(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("optimistic lock violation")]
    OptimisticLockViolation,
    #[error("query failed: {0}")]
    QueryFailed(String),
}

impl RepositoryError {
    pub fn from_sqlx_error(context: &str, err: sqlx::Error) -> Self {
        if let sqlx::Error::Database(db_err) = &err
            && let Some(code) = db_err.code()
        {
            let message = db_err.message().to_string();
            match code.as_ref() {
                // foreign_key_violation
                "23503" => {
                    return RepositoryError::ForeignKeyViolation(format!(
                        "{}: {}",
                        context, message
                    ));
                }
                // unique_violation
                "23505" => {
                    return RepositoryError::UniqueViolation(format!("{}: {}", context, message));
                }
                // string_data_right_truncation
                "22001" => {
                    return RepositoryError::Validation(format!("{}: value too long", context));
                }
                _ => {
                    return RepositoryError::QueryFailed(format!("{}: {}", context, message));
                }
            }
        }
        RepositoryError::QueryFailed(err.to_string())
    }
}

impl From<sqlx::Error> for RepositoryError {
    fn from(err: sqlx::Error) -> Self {
        RepositoryError::from_sqlx_error("sqlx", err)
    }
}
