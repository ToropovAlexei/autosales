use thiserror::Error;

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Foreign key violation: {0}")]
    ForeignKeyViolation(String),
    #[error("Unique constraint violation: {0}")]
    UniqueViolation(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Query failed")]
    QueryFailed(#[from] sqlx::Error),
}

impl RepositoryError {
    pub fn from_sqlx_error(context: &str, err: sqlx::Error) -> Self {
        if let sqlx::Error::Database(db_err) = &err {
            if let Some(code) = db_err.code() {
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
                        return RepositoryError::UniqueViolation(format!(
                            "{}: {}",
                            context, message
                        ));
                    }
                    // string_data_right_truncation
                    "22001" => {
                        return RepositoryError::Validation(format!(
                            "{}: value too long",
                            context
                        ));
                    }
                    _ => {}
                }
            }
        }
        RepositoryError::QueryFailed(err)
    }
}
