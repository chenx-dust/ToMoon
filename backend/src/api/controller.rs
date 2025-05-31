use actix_web::{web, HttpResponse, Result};
use local_ip_address::local_ip;
use serde::{Deserialize, Serialize};

use crate::{
    clash::{controller::{ClashError, ClashErrorKind, EnhancedMode}, runtime::Runtime}, subscriptions
};

use super::{ok, StatusResponse};

#[derive(Deserialize)]
pub struct DownloadSubParams {
    link: String,
}

#[derive(Serialize)]
pub struct GetConfigResponse {
    status_code: u16,
    skip_proxy: bool,
    override_dns: bool,
    enhanced_mode: EnhancedMode,
    allow_remote_access: bool,
    dashboard: String,
    secret: String,
}


pub async fn restart_clash(state: web::Data<Runtime>) -> Result<HttpResponse> {
    state.controller.write().unwrap().restart_core().await?;

    ok()
}

pub async fn reload_clash_config(state: web::Data<Runtime>) -> Result<HttpResponse> {
    let clash = state.controller.write().unwrap();
    let settings = state.settings.get();

    match clash.change_config(
        settings.skip_proxy,
        settings.override_dns,
        settings.allow_remote_access,
        settings.enhanced_mode,
        settings.dashboard.clone(),
    ) {
        Ok(_) => {}
        Err(e) => {
            log::error!("Failed while change clash config.");
            log::error!("Error Message:{}", e);
            return Err(actix_web::Error::from(ClashError {
                message: e.to_string(),
                error_kind: ClashErrorKind::KernelError,
            }));
        }
    }

    clash.reload_config().await?;

    ok()
}

pub async fn get_config(state: web::Data<Runtime>) -> Result<HttpResponse> {

    let clash = match state.controller.read() {
        Ok(x) => x,
        Err(e) => {
            log::error!("read clash_state failed: {}", e);
            return Err(actix_web::Error::from(ClashError {
                message: e.to_string(),
                error_kind: ClashErrorKind::OtherError,
            }));
        }
    };

    let settings = state.settings.get();
    let secret = match clash.get_running_secret() {
        Ok(s) => s,
        Err(_) => settings.secret.clone(),
    };

    let r = GetConfigResponse {
        skip_proxy: settings.skip_proxy,
        override_dns: settings.override_dns,
        allow_remote_access: settings.allow_remote_access,
        enhanced_mode: settings.enhanced_mode,
        dashboard: settings.dashboard.clone(),
        secret: secret,
        status_code: 200,
    };
    return Ok(HttpResponse::Ok().json(r));
}

pub async fn download_sub(
    state: web::Data<Runtime>,
    params: web::Form<DownloadSubParams>,
) -> Result<HttpResponse> {
    let url = params.link.clone();

    subscriptions::download_new_sub(&url, &state.settings).await?;

    ok()
}

pub async fn get_local_web_address() -> Result<HttpResponse> {
    match local_ip() {
        Ok(x) => {
            Ok(HttpResponse::Ok()
                .json(StatusResponse {
                    success: true,
                    data: Some(x.to_string()),
                }))
        }
        Err(_) => {
            Ok(HttpResponse::InternalServerError()
                .json(StatusResponse::<()> {
                    success: false,
                    data: None,
                }))
        }
    }
}
