use rand::{distributions::Alphanumeric, Rng};
use std::{fs, path::PathBuf, sync::{Arc, RwLock}};
use content_disposition;

use crate::{
    services::clash::controller::{ClashError, ClashErrorKind}, settings::Settings, utils::{self, get_sub_dir}
};

fn sanitize_filename(name: String) -> String {
    let name = name.replace(" ", "_");
    let name = name.replace("/", "_");
    name.split(".").next().unwrap().to_string()
}

fn gen_random_name() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(5)
        .map(char::from)
        .collect()
}

pub fn download_sub(url: String, subconv: bool, settings: Arc<RwLock<Settings>>) -> Result<String, ClashError> {
    let mut sub_name  = gen_random_name();
    let file_content: String;
    //是一个本地文件
    if let Some(local_file) = utils::get_file_path(url.clone()) {
        if subconv {
            log::warn!("download_sub: subconv is not supported for local file")
        }
        let local_file = PathBuf::from(local_file);
        if local_file.exists() {
            sub_name = String::from(local_file.file_name().unwrap().to_str().unwrap());
            sub_name = sanitize_filename(sub_name);
            file_content = match fs::read_to_string(local_file) {
                Ok(x) => x,
                Err(e) => {
                    log::error!("Failed while creating sub dir.");
                    log::error!("Error Message:{}", e);
                    return Err(ClashError {
                        message: e.to_string(),
                        error_kind: ClashErrorKind::ConfigNotFound,
                    });
                }
            };
        } else {
            log::error!("Cannt found file {}", local_file.to_str().unwrap());
            return Err(ClashError {
                message: format!("Cannt found file {}", local_file.to_str().unwrap()),
                error_kind: ClashErrorKind::InnerError,
            });
        }
    } else {
        // 是一个链接
        let url = if subconv {
            let base_url = "http://127.0.0.1:25500/sub";
            let target = "clash";
            let config = "http://127.0.0.1:55556/ACL4SSR_Online.ini";

            // 对参数进行 URL 编码
            let encoded_url = urlencoding::encode(url.as_str());
            let encoded_config = urlencoding::encode(config);

            // 构建请求 URL
            format!(
                "{}?target={}&url={}&insert=false&config={}&emoji=true&list=false&tfo=false&scv=true&fdn=false&expand=true&sort=false&new_name=true",
                base_url, target, encoded_url, encoded_config
            )
        } else {
            url.clone()
        };
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
                file_content = match x.as_str() {
                    Ok(x) => x.to_string(),
                    Err(e) => {
                        log::error!("download_sub: to_str failed with {}", e);
                        return Err(ClashError {
                            message: e.to_string(),
                            error_kind: ClashErrorKind::ConfigFormatError,
                        });
                    }
                };
                sub_name = x.headers.get("content-disposition")
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
                        sub_name
                    });
            }
            Err(e) => {
                log::error!("Failed while downloading sub.");
                log::error!("Error Message:{}", e);
                return Err(ClashError {
                    message: e.to_string(),
                    error_kind: ClashErrorKind::NetworkError,
                });
            }
        };
    }

    if !utils::check_yaml(&file_content) {
        log::error!("The downloaded subscription is not a legal profile.");
        return Err(ClashError {
            message: "The downloaded subscription is not a legal profile.".to_string(),
            error_kind: ClashErrorKind::ConfigFormatError,
        });
    }
    //保存订阅
    if let Err(e) = std::fs::create_dir_all(get_sub_dir().unwrap()) {
        log::error!("Failed while creating sub dir.");
        log::error!("Error Message:{}", e);
        return Err(ClashError {
            message: e.to_string(),
            error_kind: ClashErrorKind::InnerError,
        });
    }
    let mut filepath = get_sub_dir().unwrap().join(sub_name.clone() + ".yaml");
    if filepath.exists() {
        for i in 1..=128 {
            let new_filename = format!("{}_{}.yaml", sub_name.clone(), i);
            filepath = get_sub_dir().unwrap().join(new_filename);
            if !filepath.exists() {
                break;
            }
        }
        if filepath.exists() {
            log::error!("Failed while saving sub, cannot find a new name.");
            return Err(ClashError {
                message: "The file already exists.".to_string(),
                error_kind: ClashErrorKind::InnerError,
            });
        }
    }
    if let Err(e) = fs::write(filepath.clone(), file_content) {
        log::error!("Failed while saving sub, path: {}", filepath.to_str().unwrap());
        log::error!("Error Message:{}", e);
        return Err(ClashError {
            message: e.to_string(),
            error_kind: ClashErrorKind::InnerError,
        });
    }
    //修改下载状态
    log::info!("Download profile successfully.");
    //存入设置
    match settings.write() {
        Ok(mut x) => {
            x.subscriptions.push(crate::settings::Subscription::new(
                filepath.to_str().unwrap().to_string(),
                url.clone(),
            ));
        }
        Err(e) => {
            log::error!(
                "download_sub() faild to acquire runtime_setting write {}",
                e
            );
            return Err(ClashError {
                message: e.to_string(),
                error_kind: ClashErrorKind::InnerError,
            });
        }
    };
    Ok(sub_name)
}