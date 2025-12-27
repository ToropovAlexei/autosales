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
    QueryFailed(#[from] sqlx::Error),
}
