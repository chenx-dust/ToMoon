use rand::{distributions::Alphanumeric, Rng};
use std::{fs, io::ErrorKind, path::PathBuf};
use content_disposition;

use crate::{
    clash::{controller::{ClashError, ClashErrorKind}}, settings::{SettingsInstance, Subscription}, utils::{self, get_sub_dir}
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

async fn fetch_sub(url: &String) -> Result<(String, Option<String>), ClashError> {
    let sub_name: Option<String>;
    let file_content: String;
    //是一个本地文件
    if let Some(local_file) = utils::get_file_path(url.clone()) {
        let local_file = PathBuf::from(local_file);
        if local_file.exists() {
            sub_name = local_file.file_name()
                .and_then(|f| f.to_str())
                .and_then(|s| Some(s.to_string()));
            file_content = match tokio::fs::read_to_string(local_file).await {
                Ok(x) => x,
                Err(e) => {
                    log::error!("fetch_sub: read file failed: {}", e);
                    return Err(ClashError {
                        message: e.to_string(),
                        error_kind: match e.kind() {
                            ErrorKind::NotFound => ClashErrorKind::NotFoundError,
                            _ => ClashErrorKind::IOError,
                        },
                    });
                }
            };
        } else {
            log::error!("Cannt found file {}", local_file.to_str().unwrap());
            return Err(ClashError {
                message: format!("Cannt found file {}", local_file.to_str().unwrap()),
                error_kind: ClashErrorKind::IOError,
            });
        }
    } else {
        let response = minreq::get(url.clone())
            .with_header("User-Agent", utils::get_user_agent())
            .send().await
            .map_err(|e| ClashError {
                message: e.to_string(),
                error_kind: ClashErrorKind::NetworkError,
            })?;
        
        match response.status_code {
            200 => (),
            404 => {
                return Err(ClashError {
                    message: "Subscription not found".to_string(),
                    error_kind: ClashErrorKind::NotFoundError,
                })
            },
            c => {
                return Err(ClashError {
                    message: format!("Status Code: {}", c),
                    error_kind: ClashErrorKind::NetworkError,
                });
            },
        };
        sub_name = response.headers.get("content-disposition")
            .and_then(|header| {
                // 尝试从 content-disposition 头部获取文件名
                content_disposition::parse_content_disposition(header).filename_full()
            })
            .filter(|s| !s.is_empty())
            .or_else(|| {
                // 如果 content-disposition 头部中没有文件名，则尝试从 URL 中获取
                log::info!("Failed to get content-disposition, using url instead.");
                url.rsplit('/').next()
                    .and_then(|last_part| last_part.split('?').next()).map(|s| s.to_string())
            });
    
        file_content = response.as_str().map_err(|e| ClashError {
            message: e.to_string(),
            error_kind: ClashErrorKind::ContentError,
        })?.to_string();
    }
    let sub_name = sub_name.map(|s| sanitize_filename(s));
    if !utils::check_yaml(&file_content) {
        log::error!("The downloaded subscription is not a legal profile.");
        return Err(ClashError {
            message: "The downloaded subscription is not a legal profile.".to_string(),
            error_kind: ClashErrorKind::ContentError,
        })
    }
    Ok((file_content, sub_name))
}

async fn update_sub(sub: &Subscription) -> Result<(), ClashError> {
    let (content, _) = match fetch_sub(&sub.url).await {
        Ok(x) => x,
        Err(e) => {
            log::error!("Failed while updating sub.");
            log::error!("Error Message:{}", e);
            return Err(e);
        }
    };

    tokio::fs::write(&sub.path, content).await.map_err(|e| ClashError {
        message: e.to_string(),
        error_kind: ClashErrorKind::IOError,
    })?;

    Ok(())
}

pub async fn download_new_sub(url: &String, settings: &SettingsInstance) -> Result<String, ClashError> {
    let (file_content, sub_name) = match fetch_sub(&url).await {
        Ok(x) => x,
        Err(e) => {
            log::error!("Failed while fetching sub.");
            log::error!("Error Message:{}", e);
            return Err(e);
        }
    };
    let sub_name = sub_name.unwrap_or_else(|| gen_random_name());
    //保存订阅
    if let Err(e) = std::fs::create_dir_all(get_sub_dir().unwrap()) {
        log::error!("Failed while creating sub dir.");
        log::error!("Error Message:{}", e);
        return Err(ClashError {
            message: e.to_string(),
            error_kind: ClashErrorKind::IOError,
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
                error_kind: ClashErrorKind::IOError,
            });
        }
    }
    if let Err(e) = fs::write(filepath.clone(), file_content) {
        log::error!("Failed while saving sub, path: {}", filepath.to_str().unwrap());
        log::error!("Error Message:{}", e);
        return Err(ClashError {
            message: e.to_string(),
            error_kind: ClashErrorKind::IOError,
        });
    }
    //修改下载状态
    log::info!("Download profile successfully.");
    //存入设置
    settings.update(|mut x|
        x.subscriptions.push(crate::settings::Subscription::new(
            filepath.to_str().unwrap().to_string(),
            url.clone())
        ))
        .map_err(|e| ClashError {
                message: e.to_string(),
            error_kind: ClashErrorKind::NotFoundError,
        })?;
    Ok(sub_name)
}

pub async fn update_subs(subs: Vec<Subscription>) {
    for i in subs {
        // 异步任务
        match update_sub(&i).await {
            Ok(_) => log::info!("Subscription {} updated.", i.path),
            Err(e) => log::error!("Error updating subscription: {}", e),
        }
    }
}
