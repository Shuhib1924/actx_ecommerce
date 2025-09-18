use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use std::fmt;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    
    #[error("Not found")]
    NotFound,
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Internal server error")]
    InternalError,
    
    #[error("Session error")]
    SessionError,
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        let message = self.to_string();
        
        HttpResponse::build(status).json(serde_json::json!({
            "error": message,
            "status": status.as_u16()
        }))
    }
    
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::SessionError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub type Result<T> = std::result::Result<T, AppError>;