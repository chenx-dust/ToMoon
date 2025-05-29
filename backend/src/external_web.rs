use actix_web::{body::BoxBody, web, HttpResponse, Result};
use local_ip_address::local_ip;
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf, sync::Mutex};
use content_disposition;

use crate::{
    services::clash::{ClashError, ClashErrorKind, EnhancedMode},
    utils,
    settings::State,
};

pub struct Runtime(pub *const crate::services::clash::ClashRuntime);
unsafe impl Send for Runtime {}

pub struct AppState {
    pub link_table: Mutex<HashMap<u16, String>>,
    pub runtime: Mutex<Runtime>,
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
    let runtime_state;
    unsafe {
        let runtime = runtime.0.as_ref().unwrap();
        runtime_settings = runtime.settings_clone();
        runtime_state = runtime.state_clone();
    }
    match runtime_settings.write() {
        Ok(mut x) => {
            x.skip_proxy = skip_proxy;
            let mut state = match runtime_state.write() {
                Ok(x) => x,
                Err(e) => {
                    log::error!("set_enable failed to acquire state write lock: {}", e);
                    return Err(actix_web::Error::from(ClashError {
                        message: e.to_string(),
                        error_kind: ClashErrorKind::InnerError,
                    }));
                }
            };
            state.dirty = true;
        }
        Err(e) => {
            log::error!("Failed while toggle skip Steam proxy.");
            log::error!("Error Message:{}", e);
            return Err(actix_web::Error::from(ClashError {
                message: e.to_string(),
                error_kind: ClashErrorKind::ConfigNotFound,
            }));
        }
    }
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
    let runtime_state;
    unsafe {
        let runtime = runtime.0.as_ref().unwrap();
        runtime_settings = runtime.settings_clone();
        runtime_state = runtime.state_clone();
    }
    match runtime_settings.write() {
        Ok(mut x) => {
            x.override_dns = override_dns;
            let mut state = match runtime_state.write() {
                Ok(x) => x,
                Err(e) => {
                    log::error!("override_dns failed to acquire state write lock: {}", e);
                    return Err(actix_web::Error::from(ClashError {
                        message: e.to_string(),
                        error_kind: ClashErrorKind::InnerError,
                    }));
                }
            };
            state.dirty = true;
        }
        Err(e) => {
            log::error!("Failed while toggle override dns.");
            log::error!("Error Message:{}", e);
            return Err(actix_web::Error::from(ClashError {
                message: e.to_string(),
                error_kind: ClashErrorKind::ConfigNotFound,
            }));
        }
    }
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
    let runtime_state;
    unsafe {
        let runtime = runtime.0.as_ref().unwrap();
        runtime_settings = runtime.settings_clone();
        runtime_state = runtime.state_clone();
    }
    match runtime_settings.write() {
        Ok(mut x) => {
            x.allow_remote_access = allow_remote_access;
            let mut state = match runtime_state.write() {
                Ok(x) => x,
                Err(e) => {
                    log::error!(
                        "allow_remote_access failed to acquire state write lock: {}",
                        e
                    );
                    return Err(actix_web::Error::from(ClashError {
                        message: e.to_string(),
                        error_kind: ClashErrorKind::InnerError,
                    }));
                }
            };
            state.dirty = true;
        }
        Err(e) => {
            log::error!("Failed while toggle allow_remote_access.");
            log::error!("Error Message:{}", e);
            return Err(actix_web::Error::from(ClashError {
                message: e.to_string(),
                error_kind: ClashErrorKind::ConfigNotFound,
            }));
        }
    }
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
    let runtime_state;
    unsafe {
        let runtime = runtime.0.as_ref().unwrap();
        runtime_settings = runtime.settings_clone();
        runtime_state = runtime.state_clone();
    }
    match runtime_settings.write() {
        Ok(mut x) => {
            x.enhanced_mode = enhanced_mode;
            let mut state = match runtime_state.write() {
                Ok(x) => x,
                Err(e) => {
                    log::error!("enhanced_mode failed to acquire state write lock: {}", e);
                    return Err(actix_web::Error::from(ClashError {
                        message: e.to_string(),
                        error_kind: ClashErrorKind::InnerError,
                    }));
                }
            };
            state.dirty = true;
        }
        Err(e) => {
            log::error!("Failed while toggle enhanced mode.");
            log::error!("Error Message:{}", e);
            return Err(actix_web::Error::from(ClashError {
                message: e.to_string(),
                error_kind: ClashErrorKind::ConfigNotFound,
            }));
        }
    }
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
    let runtime_state;
    unsafe {
        let runtime = runtime.0.as_ref().unwrap();
        runtime_settings = runtime.settings_clone();
        runtime_state = runtime.state_clone();
    }

    match runtime_settings.write() {
        Ok(mut x) => {
            x.dashboard = dashboard;
            let mut state = match runtime_state.write() {
                Ok(x) => x,
                Err(e) => {
                    log::error!("set_dashboard failed to acquire state write lock: {}", e);
                    return Err(actix_web::Error::from(ClashError {
                        message: e.to_string(),
                        error_kind: ClashErrorKind::InnerError,
                    }));
                }
            };
            state.dirty = true;
        }
        Err(e) => {
            log::error!("Failed while set dashboard.");
            log::error!("Error Message:{}", e);
            return Err(actix_web::Error::from(ClashError {
                message: e.to_string(),
                error_kind: ClashErrorKind::ConfigNotFound,
            }));
        }
    }
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

    // let settings = match runtime_settings.write() {
    //     Ok(x) => x,
    //     Err(e) => {
    //         log::error!(
    //             "read runtime_settings failed to acquire state write lock: {}",
    //             e
    //         );
    //         return Err(actix_web::Error::from(ClashError {
    //             Message: e.to_string(),
    //             ErrorKind: ClashErrorKind::InnerError,
    //         }));
    //     }
    // };

    // match clash.change_config(
    //     settings.skip_proxy,
    //     settings.override_dns,
    //     settings.allow_remote_access,
    //     settings.enhanced_mode,
    //     settings.dashboard.clone(),
    // ) {
    //     Ok(_) => {}
    //     Err(e) => {
    //         log::error!("Failed while change clash config.");
    //         log::error!("Error Message:{}", e);
    //         return Err(actix_web::Error::from(ClashError {
    //             Message: e.to_string(),
    //             ErrorKind: ClashErrorKind::InnerError,
    //         }));
    //     }
    // }

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

    let settings = match runtime_settings.write() {
        Ok(x) => x,
        Err(e) => {
            log::error!("read runtime_settings failed: {}", e);
            return Err(actix_web::Error::from(ClashError {
                message: e.to_string(),
                error_kind: ClashErrorKind::InnerError,
            }));
        }
    };

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

    match runtime_settings.read() {
        Ok(x) => {
            let secret = match clash.get_running_secret() {
                Ok(s) => s,
                Err(_) => x.secret.clone(),
            };

            let r = GetConfigResponse {
                skip_proxy: x.skip_proxy,
                override_dns: x.override_dns,
                allow_remote_access: x.allow_remote_access,
                enhanced_mode: x.enhanced_mode,
                dashboard: x.dashboard.clone(),
                secret: secret,
                status_code: 200,
            };
            return Ok(HttpResponse::Ok().json(r));
        }
        Err(e) => {
            log::error!("Failed while geting skip Steam proxy.");
            log::error!("Error Message:{}", e);
            return Err(actix_web::Error::from(ClashError {
                message: e.to_string(),
                error_kind: ClashErrorKind::ConfigNotFound,
            }));
        }
    };
}

pub async fn download_sub(
    state: web::Data<AppState>,
    params: web::Form<GenLinkParams>,
) -> Result<HttpResponse> {
    let mut url = params.link.clone();
    let subconv = params.subconv.clone();
    let runtime = state.runtime.lock().unwrap();

    let runtime_settings;
    let runtime_state;
    unsafe {
        let runtime = runtime.0.as_ref().unwrap();
        runtime_settings = runtime.settings_clone();
        runtime_state = runtime.state_clone();
    }

    let home = match runtime_state.read() {
        Ok(state) => state.home.clone(),
        Err(_) => State::default().home,
    };
    let path: PathBuf = home.join(".config/tomoon/subs/");

    //是一个本地文件
    if let Some(local_file) = utils::get_file_path(url.clone()) {
        let local_file = PathBuf::from(local_file);
        let filename = (|| -> Result<String, ()> {
            // 如果文件名可被读取则采用
            let mut filename = String::from(local_file.file_name().ok_or(())?.to_str().ok_or(())?);
            if !filename.to_lowercase().ends_with(".yaml")
                && !filename.to_lowercase().ends_with(".yml")
            {
                filename += ".yaml";
            }
            Ok(filename)
        })()
        .unwrap_or({
            log::warn!("The subscription does not have a proper file name.");
            // 否则采用随机名字
            rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(5)
                .map(char::from)
                .collect::<String>()
                + ".yaml"
        });
        if local_file.exists() {
            let file_content = match fs::read_to_string(local_file) {
                Ok(x) => x,
                Err(e) => {
                    log::error!("Failed while creating sub dir.");
                    log::error!("Error Message:{}", e);
                    return Err(actix_web::Error::from(ClashError {
                        message: e.to_string(),
                        error_kind: ClashErrorKind::ConfigNotFound,
                    }));
                }
            };
            if !utils::check_yaml(&file_content) {
                log::error!("The downloaded subscription is not a legal profile.");
                return Err(actix_web::Error::from(ClashError {
                    message: "The downloaded subscription is not a legal profile.".to_string(),
                    error_kind: ClashErrorKind::ConfigFormatError,
                }));
            }
            //保存订阅
            let path = path.join(filename);
            if let Some(parent) = path.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    log::error!("Failed while creating sub dir.");
                    log::error!("Error Message:{}", e);
                    return Err(actix_web::Error::from(ClashError {
                        message: e.to_string(),
                        error_kind: ClashErrorKind::InnerError,
                    }));
                }
            }
            let path = path.to_str().unwrap();
            if let Err(e) = fs::write(path, file_content) {
                log::error!("Failed while saving sub, path: {}", path);
                log::error!("Error Message:{}", e);
                return Err(actix_web::Error::from(ClashError {
                    message: e.to_string(),
                    error_kind: ClashErrorKind::InnerError,
                }));
            }
            //修改下载状态
            log::info!("Download profile successfully.");
            //存入设置
            match runtime_settings.write() {
                Ok(mut x) => {
                    x.subscriptions.push(crate::settings::Subscription::new(
                        path.to_string(),
                        url.clone(),
                    ));
                    let mut state = match runtime_state.write() {
                        Ok(x) => x,
                        Err(e) => {
                            log::error!("set_enable failed to acquire state write lock: {}", e);
                            return Err(actix_web::Error::from(ClashError {
                                message: e.to_string(),
                                error_kind: ClashErrorKind::InnerError,
                            }));
                        }
                    };
                    state.dirty = true;
                }
                Err(e) => {
                    log::error!(
                        "download_sub() faild to acquire runtime_setting write {}",
                        e
                    );
                    return Err(actix_web::Error::from(ClashError {
                        message: e.to_string(),
                        error_kind: ClashErrorKind::InnerError,
                    }));
                }
            };
        } else {
            log::error!("Cannt found file {}", local_file.to_str().unwrap());
            return Err(actix_web::Error::from(ClashError {
                message: format!("Cannt found file {}", local_file.to_str().unwrap()),
                error_kind: ClashErrorKind::InnerError,
            }));
        }
        // 是一个链接
    } else {
        if subconv {
            let base_url = "http://127.0.0.1:25500/sub";
            let target = "clash";
            let config = "http://127.0.0.1:55556/ACL4SSR_Online.ini";

            // 对参数进行 URL 编码
            let encoded_url = urlencoding::encode(url.as_str());
            let encoded_config = urlencoding::encode(config);

            // 构建请求 URL
            url = format!(
                "{}?target={}&url={}&insert=false&config={}&emoji=true&list=false&tfo=false&scv=true&fdn=false&expand=true&sort=false&new_name=true",
                base_url, target, encoded_url, encoded_config
            );
        }
        match minreq::get(url.clone())
            .with_header(
                "User-Agent",
                format!(
                    "ToMoon/{} mihomo/1.19.4 clash-verge/2.2.3 Clash/v1.18.0",
                    env!("CARGO_PKG_VERSION")
                ),
            )
            .with_timeout(120)
            .send()
        {
            Ok(x) => {
                let response = x.as_str().unwrap();

                if !utils::check_yaml(&String::from(response)) {
                    log::error!("The downloaded subscription is not a legal profile.");
                    return Err(actix_web::Error::from(ClashError {
                        message: "The downloaded subscription is not a legal profile.".to_string(),
                        error_kind: ClashErrorKind::ConfigFormatError,
                    }));
                }
                let filename = x.headers.get("content-disposition")
                    .and_then(|header| {
                        // 尝试从 content-disposition 头部获取文件名
                        // header.split("filename=").nth(1)
                        //     .and_then(|s| s.split(';').next())
                        //     .map(|s| s.trim_matches('"'))
                        content_disposition::parse_content_disposition(header).filename_full()
                    })
                    .filter(|s| !s.is_empty())
                    .or_else(|| {
                        // 如果 content-disposition 头部中没有文件名，则尝试从 URL 中获取
                        log::info!("Failed to get content-disposition, using url instead.");
                        url.rsplit('/').next()
                            .and_then(|last_part| last_part.split('?').next()).map(|s| s.to_string())
                    })
                    .unwrap_or_else(|| {
                        // 如果 URL 中没有文件名，则生成一个随机文件名
                        log::warn!("The downloaded subscription does not have a file name.");
                        gen_random_name()
                    });
                let filename = match filename.to_ascii_lowercase() {
                    ref lower if lower.ends_with(".yaml") || lower.ends_with(".yml") => filename,
                    _ => filename + ".yaml",
                };
                let mut filepath = path.join(filename.clone());
                if filepath.exists() {
                    for i in 1..=128 {
                        let new_filename = format!("{}_{}.yaml", filename.trim_end_matches(".yaml"), i);
                        filepath = path.join(new_filename);
                        if !filepath.exists() {
                            break;
                        }
                    }
                    if filepath.exists() {
                        log::error!("Failed while saving sub, cannot find a new name.");
                        return Err(actix_web::Error::from(ClashError {
                            message: "The file already exists.".to_string(),
                            error_kind: ClashErrorKind::InnerError,
                        }));
                    }
                }
                //保存订阅
                if let Some(parent) = filepath.parent() {
                    if let Err(e) = std::fs::create_dir_all(parent) {
                        log::error!("Failed while creating sub dir.");
                        log::error!("Error Message:{}", e);
                        return Err(actix_web::Error::from(ClashError {
                            message: e.to_string(),
                            error_kind: ClashErrorKind::InnerError,
                        }));
                    }
                }
                let path = filepath.to_str().unwrap();
                log::info!("Writing to path: {}", path);
                if let Err(e) = fs::write(path, response) {
                    log::error!("Failed while saving sub.");
                    log::error!("Error Message:{}", e);
                    return Err(actix_web::Error::from(ClashError {
                        message: e.to_string(),
                        error_kind: ClashErrorKind::InnerError,
                    }));
                }
                //下载成功
                //修改下载状态
                log::info!("Download profile successfully.");
                //存入设置
                match runtime_settings.write() {
                    Ok(mut x) => {
                        x.subscriptions
                            .push(crate::settings::Subscription::new(path.to_string(), url));
                        let mut state = match runtime_state.write() {
                            Ok(x) => x,
                            Err(e) => {
                                log::error!("set_enable failed to acquire state write lock: {}", e);
                                return Err(actix_web::Error::from(ClashError {
                                    message: e.to_string(),
                                    error_kind: ClashErrorKind::InnerError,
                                }));
                            }
                        };
                        state.dirty = true;
                    }
                    Err(e) => {
                        log::error!(
                            "download_sub() faild to acquire runtime_setting write {}",
                            e
                        );
                        return Err(actix_web::Error::from(ClashError {
                            message: e.to_string(),
                            error_kind: ClashErrorKind::InnerError,
                        }));
                    }
                }
            }
            Err(e) => {
                log::error!("Failed while downloading sub.");
                log::error!("Error Message:{}", e);
                return Err(actix_web::Error::from(ClashError {
                    message: e.to_string(),
                    error_kind: ClashErrorKind::NetworkError,
                }));
            }
        };
    }
    let r = GenLinkResponse {
        message: "下载成功".to_string(),
        status_code: 200,
    };
    Ok(HttpResponse::Ok().json(r))
}

fn gen_random_name() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(5)
        .map(char::from)
        .collect()
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
