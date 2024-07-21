use std::fmt::Display;

use actix_web::{HttpResponse, ResponseError};

#[derive(Debug)]
pub enum ServiceError {
    InternalServerError,
    Unauthorized,
    BadRequest,
}

impl Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceError::InternalServerError => write!(f, "Internal server error"),
            ServiceError::Unauthorized => write!(f, "Unauthorized"),
            ServiceError::BadRequest => write!(f, "Bad request"),
        }
    }
}

impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServiceError::InternalServerError => HttpResponse::InternalServerError().finish(),
            ServiceError::Unauthorized => HttpResponse::Unauthorized().finish(),
            ServiceError::BadRequest => HttpResponse::BadRequest().finish(),
        }
    }
}
