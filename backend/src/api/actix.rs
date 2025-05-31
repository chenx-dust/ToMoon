use actix_web::{body::BoxBody, web, HttpResponse, Result};
use local_ip_address::local_ip;
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf, sync::Mutex};
use content_disposition;

use crate::{
    services::clash::controller::{ClashError, ClashErrorKind, EnhancedMode}, subscriptions, utils
};

pub struct RuntimePtr(pub *const crate::services::clash::runtime::Runtime);
unsafe impl Send for RuntimePtr {}

pub struct AppState {
    pub link_table: Mutex<HashMap<u16, String>>,
    pub runtime: Mutex<RuntimePtr>,
}

#[derive(Deserialize)]
pub struct GenLinkParams {
    link: String,
    subconv: bool,
}

#[derive(Deserialize)]
pub struct SkipProxyParams {
    skip_proxy: bool,
}

#[derive(Deserialize)]
pub struct AllowRemoteAccessParams {
    allow_remote_access: bool,
}

#[derive(Deserialize)]
pub struct OverrideDNSParams {
    override_dns: bool,
}

#[derive(Deserialize)]
pub struct EnhancedModeParams {
    enhanced_mode: EnhancedMode,
}

#[derive(Deserialize)]
pub struct DashboardParams {
    dashboard: String,
}

#[derive(Serialize, Deserialize)]
pub struct GenLinkResponse {
    status_code: u16,
    message: String,
}

#[derive(Serialize, Deserialize)]
pub struct SkipProxyResponse {
    status_code: u16,
    message: String,
}

#[derive(Serialize, Deserialize)]
pub struct OverrideDNSResponse {
    status_code: u16,
    message: String,
}

#[derive(Serialize, Deserialize)]
pub struct AllowRemoteAccessResponse {
    status_code: u16,
    message: String,
}

#[derive(Serialize, Deserialize)]
pub struct DashboardResponse {
    status_code: u16,
    message: String,
}

#[derive(Deserialize)]
pub struct GetLinkParams {
    code: u16,
}
#[derive(Serialize, Deserialize)]
pub struct GetLinkResponse {
    status_code: u16,
    link: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct GetConfigResponse {
    status_code: u16,
    skip_proxy: bool,
    override_dns: bool,
    enhanced_mode: EnhancedMode,
    allow_remote_access: bool,
    dashboard: String,
    secret: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetLocalIpAddressResponse {
    status_code: u16,
    ip: Option<String>,
}

impl actix_web::ResponseError for ClashError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        if self.error_kind == ClashErrorKind::ConfigNotFound {
            actix_web::http::StatusCode::NOT_FOUND
        } else {
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
        }
    }

    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        let mut res = HttpResponse::new(self.status_code());
        let mime = "text/plain; charset=utf-8";
        res.headers_mut().insert(
            actix_web::http::header::CONTENT_TYPE,
            actix_web::http::header::HeaderValue::from_str(mime).unwrap(),
        );
        res.set_body(BoxBody::new(self.message.clone()))
    }
}

pub async fn skip_proxy(
    state: web::Data<AppState>,
    params: web::Form<SkipProxyParams>,
) -> Result<HttpResponse> {
    let skip_proxy = params.skip_proxy.clone();
    let runtime = state.runtime.lock().unwrap();
    let runtime_settings;
    unsafe {
        let runtime = runtime.0.as_ref().unwrap();
        runtime_settings = runtime.settings_clone();
    }
    runtime_settings.update(|mut x| x.skip_proxy = skip_proxy)
        .map_err(|e| actix_web::Error::from(ClashError {
             message: e.to_string(),
            error_kind: ClashErrorKind::ConfigNotFound,
        }))?;

    let r = SkipProxyResponse {
        message: "修改成功".to_string(),
        status_code: 200,
    };
    Ok(HttpResponse::Ok().json(r))
}

pub async fn override_dns(
    state: web::Data<AppState>,
    params: web::Form<OverrideDNSParams>,
) -> Result<HttpResponse> {
    let override_dns = params.override_dns.clone();
    let runtime = state.runtime.lock().unwrap();
    let runtime_settings;
    unsafe {
        let runtime = runtime.0.as_ref().unwrap();
        runtime_settings = runtime.settings_clone();
    }
    runtime_settings.update(|mut x| x.override_dns = override_dns)
        .map_err(|e| actix_web::Error::from(ClashError {
             message: e.to_string(),
            error_kind: ClashErrorKind::ConfigNotFound,
        }))?;

    let r = OverrideDNSResponse {
        message: "修改成功".to_string(),
        status_code: 200,
    };
    Ok(HttpResponse::Ok().json(r))
}

// allow_remote_access
pub async fn allow_remote_access(
    state: web::Data<AppState>,
    params: web::Form<AllowRemoteAccessParams>,
) -> Result<HttpResponse> {
    let allow_remote_access = params.allow_remote_access.clone();
    let runtime = state.runtime.lock().unwrap();
    let runtime_settings;
    unsafe {
        let runtime = runtime.0.as_ref().unwrap();
        runtime_settings = runtime.settings_clone();
    }
    runtime_settings.update(|mut x| x.allow_remote_access = allow_remote_access)
        .map_err(|e| actix_web::Error::from(ClashError {
             message: e.to_string(),
            error_kind: ClashErrorKind::ConfigNotFound,
        }))?;

    let r = OverrideDNSResponse {
        message: "修改成功".to_string(),
        status_code: 200,
    };
    Ok(HttpResponse::Ok().json(r))
}

pub async fn enhanced_mode(
    state: web::Data<AppState>,
    params: web::Form<EnhancedModeParams>,
) -> Result<HttpResponse> {
    let enhanced_mode = params.enhanced_mode.clone();
    let runtime = state.runtime.lock().unwrap();
    let runtime_settings;
    unsafe {
        let runtime = runtime.0.as_ref().unwrap();
        runtime_settings = runtime.settings_clone();
    }
    runtime_settings.update(|mut x| x.enhanced_mode = enhanced_mode)
        .map_err(|e| actix_web::Error::from(ClashError {
             message: e.to_string(),
            error_kind: ClashErrorKind::ConfigNotFound,
        }))?;
    let r = OverrideDNSResponse {
        message: "修改成功".to_string(),
        status_code: 200,
    };
    Ok(HttpResponse::Ok().json(r))
}

// set_dashboard
pub async fn set_dashboard(
    state: web::Data<AppState>,
    params: web::Form<DashboardParams>,
) -> Result<HttpResponse> {
    let dashboard = params.dashboard.clone();
    let runtime = state.runtime.lock().unwrap();
    let runtime_settings;
    unsafe {
        let runtime = runtime.0.as_ref().unwrap();
        runtime_settings = runtime.settings_clone();
    }

    runtime_settings.update(|mut x| x.dashboard = dashboard.clone())
        .map_err(|e| actix_web::Error::from(ClashError {
             message: e.to_string(),
            error_kind: ClashErrorKind::ConfigNotFound,
        }))?;
    let r = DashboardResponse {
        message: "修改成功".to_string(),
        status_code: 200,
    };
    Ok(HttpResponse::Ok().json(r))
}

pub async fn restart_clash(state: web::Data<AppState>) -> Result<HttpResponse> {
    let runtime = state.runtime.lock().unwrap();
    // let runtime_settings;
    let clash_state;
    unsafe {
        let runtime = runtime.0.as_ref().unwrap();
        // runtime_settings = runtime.settings_clone();
        clash_state = runtime.clash_state_clone();
    }

    let clash = match clash_state.write() {
        Ok(x) => x,
        Err(e) => {
            log::error!("read clash_state failed to acquire state write lock: {}", e);
            return Err(actix_web::Error::from(ClashError {
                message: e.to_string(),
                error_kind: ClashErrorKind::InnerError,
            }));
        }
    };

    match clash.restart_core().await {
        Ok(_) => {}
        Err(e) => {
            log::error!("Failed while restart clash.");
            log::error!("Error Message:{}", e);
            return Err(actix_web::Error::from(ClashError {
                message: e.to_string(),
                error_kind: ClashErrorKind::InnerError,
            }));
        }
    }

    let r = GenLinkResponse {
        message: "重启成功".to_string(),
        status_code: 200,
    };
    Ok(HttpResponse::Ok().json(r))
}

pub async fn reload_clash_config(state: web::Data<AppState>) -> Result<HttpResponse> {
    let runtime = state.runtime.lock().unwrap();
    let runtime_settings;
    let clash_state;
    unsafe {
        let runtime = runtime.0.as_ref().unwrap();
        runtime_settings = runtime.settings_clone();
        clash_state = runtime.clash_state_clone();
    }

    let clash = match clash_state.write() {
        Ok(x) => x,
        Err(e) => {
            log::error!("read clash_state failed: {}", e);
            return Err(actix_web::Error::from(ClashError {
                message: e.to_string(),
                error_kind: ClashErrorKind::InnerError,
            }));
        }
    };


    let settings = runtime_settings.get();

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
                error_kind: ClashErrorKind::InnerError,
            }));
        }
    }

    match clash.reload_config().await {
        Ok(_) => {}
        Err(e) => {
            log::error!("Failed while reload clash config.");
            log::error!("Error Message:{}", e);
            return Err(actix_web::Error::from(ClashError {
                message: e.to_string(),
                error_kind: ClashErrorKind::InnerError,
            }));
        }
    }

    let r = GenLinkResponse {
        message: "重载成功".to_string(),
        status_code: 200,
    };
    Ok(HttpResponse::Ok().json(r))
}

pub async fn get_config(state: web::Data<AppState>) -> Result<HttpResponse> {
    let runtime = state.runtime.lock().unwrap();
    let runtime_settings;
    let clash_state;
    unsafe {
        let runtime = runtime.0.as_ref().unwrap();
        runtime_settings = runtime.settings_clone();
        clash_state = runtime.clash_state_clone();
    }

    let clash = match clash_state.read() {
        Ok(x) => x,
        Err(e) => {
            log::error!("read clash_state failed: {}", e);
            return Err(actix_web::Error::from(ClashError {
                message: e.to_string(),
                error_kind: ClashErrorKind::InnerError,
            }));
        }
    };

    let settings = runtime_settings.get();
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
    state: web::Data<AppState>,
    params: web::Form<GenLinkParams>,
) -> Result<HttpResponse> {
    let url = params.link.clone();
    let subconv = params.subconv.clone();
    let runtime = state.runtime.lock().unwrap();

    let runtime_settings;
    unsafe {
        let runtime = runtime.0.as_ref().unwrap();
        runtime_settings = runtime.settings_clone();
    }

    subscriptions::download_sub(url, subconv, runtime_settings)?;

    let r = GenLinkResponse {
        message: "下载成功".to_string(),
        status_code: 200,
    };
    Ok(HttpResponse::Ok().json(r))
}


pub async fn get_link(
    state: web::Data<AppState>,
    info: web::Query<GetLinkParams>,
) -> Result<web::Json<GetLinkResponse>> {
    let table = state.link_table.lock().unwrap();
    let link = table.get(&info.code);
    match link {
        Some(x) => {
            let r = GetLinkResponse {
                link: Some((*x).clone()),
                status_code: 200,
            };
            return Ok(web::Json(r));
        }
        None => {
            let r = GetLinkResponse {
                link: None,
                status_code: 404,
            };
            return Ok(web::Json(r));
        }
    }
}

pub async fn get_local_web_address() -> Result<HttpResponse> {
    match local_ip() {
        Ok(x) => {
            let r = GetLocalIpAddressResponse {
                status_code: 200,
                ip: Some(x.to_string()),
            };
            return Ok(HttpResponse::Ok().json(r));
        }
        Err(_) => {
            let r = GetLocalIpAddressResponse {
                status_code: 404,
                ip: None,
            };
            return Ok(HttpResponse::Ok().json(r));
        }
    };
}
