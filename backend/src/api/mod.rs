pub mod settings;
pub mod controller;

use actix_web::{HttpResponse, Result};
use serde::{Deserialize, Serialize};

use crate::{
    clash::controller::{ClashError, ClashErrorKind}, settings::SettingsError
};

#[derive(Deserialize)]
pub struct SingleParam<T> {
    param: T,
}

#[derive(Serialize)]
pub struct StatusResponse<T> {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
}

impl actix_web::ResponseError for ClashError {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        match self.error_kind {
            ClashErrorKind::NotFoundError => HttpResponse::NotFound(),
            ClashErrorKind::ContentError => HttpResponse::BadRequest(),
            _ => HttpResponse::InternalServerError(),
        }.json(StatusResponse {
            success: false,
            data: Some(self.message.clone()),
        })
    }
}

impl actix_web::ResponseError for SettingsError {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::InternalServerError().json(StatusResponse {
            success: false,
            data: Some(self.to_string()),
        })
    }
}

fn ok() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(StatusResponse::<()> {
        success: true,
        data: None,
    }))
}