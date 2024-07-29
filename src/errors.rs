use std::fmt::Display;

use actix_web::{HttpResponse, ResponseError};

#[derive(Debug)]
pub enum ServiceError {
    InternalServerError(String),
    Unauthorized(String),
    BadRequest(String),
    NotFound(String),
}

impl Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceError::InternalServerError(msg) => write!(f, "Erro interno: {}", msg),
            ServiceError::Unauthorized(msg) => write!(f, "Não autorizado: {}", msg),
            ServiceError::BadRequest(msg) => write!(f, "Requisição inválida: {}", msg),
            ServiceError::NotFound(msg) => write!(f, "Não encontrado: {}", msg),
        }
    }
}

impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServiceError::InternalServerError(msg) => {
                HttpResponse::InternalServerError().body(format!("{}", msg))
            }
            ServiceError::Unauthorized(msg) => {
                HttpResponse::Unauthorized().body(format!("{}", msg))
            }
            ServiceError::BadRequest(msg) => HttpResponse::BadRequest().body(format!("{}", msg)),
            ServiceError::NotFound(msg) => HttpResponse::NotFound().body(format!("{}", msg)),
        }
    }
}
