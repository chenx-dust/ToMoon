use std::fs;

use crate::{
    clash::runtime::{DownloadStatus, RunningStatus}, subscriptions, utils
};

use crate::clash::runtime::Runtime;

use usdpl_back::{core::serdes::Primitive, AsyncCallable};

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
pub const NAME: &'static str = env!("CARGO_PKG_NAME");

pub fn get_clash_status(_: Vec<Primitive>) -> Vec<Primitive> {
    let is_clash_running = utils::is_clash_running();
    log::debug!("get_enable() success");
    log::info!("get clash status with {}", is_clash_running);
    vec![is_clash_running.into()]
}

pub fn set_clash_status(runtime: &Runtime) -> impl Fn(Vec<Primitive>) -> Vec<Primitive> {
    let runtime_settings = runtime.settings_clone();
    let clash = runtime.clash_state_clone();
    let running_status = runtime.running_status_clone();
    move |params| {
        let Some(Primitive::Bool(enabled)) = params.get(0) else {
            log::error!("set_clash_status: invalid params");
            return vec![false.into()];
        };
        let settings = runtime_settings.get();
        log::info!("set_clash_status: setting to {}", enabled);
        let mut clash = match clash.write() {
            Ok(x) => x,
            Err(e) => {
                log::error!("set_enable failed to acquire state write lock: {}", e);
                return vec![false.into()];
            }
        };
        let mut run_status = match running_status.write() {
            Ok(x) => x,
            Err(e) => {
                log::error!("set_enable failed to acquire run status write lock: {}", e);
                return vec![false.into()];
            }
        };
        *run_status = RunningStatus::Loading;
        // 有些时候第一次没有选择订阅
        if settings.current_sub == "" {
            log::info!("set_clash_status: no profile provided, try to use first profile.");
            if let Some(sub) = settings.subscriptions.get(0) {
                if let Err(e) = runtime_settings.update(|mut x| x.current_sub = sub.path.clone()) {
                    log::error!("set_clash_status: error: {}", e);
                    *run_status = RunningStatus::Failed;
                    return vec![false.into()];
                }
            } else {
                log::error!("no profile provided.");
                *run_status = RunningStatus::Failed;
                return vec![false.into()];
            }
        }
        if *enabled {
            match clash.run(
                &settings.current_sub,
                settings.skip_proxy,
                settings.override_dns,
                settings.allow_remote_access,
                settings.enhanced_mode,
                settings.dashboard.clone(),
                ) {
                Ok(_) => (),
                Err(e) => {
                    log::error!("Run clash error: {}", e);
                    *run_status = RunningStatus::Failed;
                    return vec![false.into()];
                }
            }
        } else {
            // Disable Clash
            match clash.stop() {
                Ok(_) => {
                    log::info!("successfully disable clash");
                }
                Err(e) => {
                    log::error!("Disable clash error: {}", e);
                    *run_status = RunningStatus::Failed;
                    return vec![false.into()];
                }
            }
        }
        *run_status = RunningStatus::Success;
        log::debug!("set_enable({}) success", enabled);
        vec![(*enabled).into()]
    }
}

pub fn download_sub(runtime: &Runtime) -> impl AsyncCallable {
    let download_status = runtime.downlaod_status_clone();
    let runtime_setting = runtime.settings_clone();
    move |params: Vec<Primitive>| {
        let download_status = download_status.clone();
        let runtime_setting = runtime_setting.clone();
        async move {
            if let Some(Primitive::String(url)) = params.get(0) {
                match download_status.write() {
                    Ok(mut x) => {
                        *x = DownloadStatus::Downloading;
                        //新线程复制准备
                        let url = url.clone();
                        let download_status = download_status.clone();
                        let runtime_setting = runtime_setting.clone();
                        //开始下载
                        tokio::spawn(async move {
                            let update_status = |status: DownloadStatus| {
                                //修改下载状态
                                match download_status.write() {
                                    Ok(mut x) => {
                                        *x = status;
                                    }
                                    Err(e) => {
                                        log::error!(
                                            "download_sub() faild to acquire download_status write {}",
                                            e
                                        );
                                    }
                                }
                            };
                            match subscriptions::download_new_sub(&url, &runtime_setting).await {
                                Ok(_) => update_status(DownloadStatus::Success),
                                Err(e) => {
                                    update_status(DownloadStatus::Failed);
                                    log::error!("download_sub() failed to download sub {}", e);
                                }
                            }
                        });
                    }
                    Err(_) => {
                        log::error!("download_sub() faild to acquire state write");
                        return vec![];
                    }
                }
            }
            return vec![];
        }
    }
}

pub fn get_download_status(runtime: &Runtime) -> impl Fn(Vec<Primitive>) -> Vec<Primitive> {
    let download_status = runtime.downlaod_status_clone();
    move |_| {
        match download_status.read() {
            Ok(x) => {
                let status = x.to_string();
                return vec![status.into()];
            }
            Err(_) => {
                log::error!("Error occured while get_download_status()");
            }
        }
        return vec![];
    }
}

pub fn get_running_status(runtime: &Runtime) -> impl Fn(Vec<Primitive>) -> Vec<Primitive> {
    let running_status = runtime.running_status_clone();
    move |_| {
        match running_status.read() {
            Ok(x) => {
                let status = x.to_string();
                return vec![status.into()];
            }
            Err(_) => {
                log::error!("Error occured while get_running_status()");
            }
        }
        return vec![];
    }
}

pub fn get_sub_list(runtime: &Runtime) -> impl Fn(Vec<Primitive>) -> Vec<Primitive> {
    let runtime_setting = runtime.settings_clone();
    move |_| {
        let x = runtime_setting.get();
        match serde_json::to_string(&x.subscriptions) {
            Ok(x) => {
                //返回 json 编码的订阅
                return vec![x.into()];
            }
            Err(e) => {
                log::error!("Error while serializing data structures");
                log::error!("Error message: {}", e);
                return vec![];
            }
        }
    }
}

// get_current_sub 获取当前订阅
pub fn get_current_sub(runtime: &Runtime) -> impl Fn(Vec<Primitive>) -> Vec<Primitive> {
    let runtime_setting = runtime.settings_clone();
    move |_| vec![runtime_setting.get().current_sub.clone().into()]
}

pub fn delete_sub(runtime: &Runtime) -> impl Fn(Vec<Primitive>) -> Vec<Primitive> {
    let runtime_setting = runtime.settings_clone();
    move |params| {
        let Some(Primitive::U32(id)) = params.get(0) else {
            return vec![Primitive::Bool(false), Primitive::String(String::from("Invalid id"))];
        };
        let result = runtime_setting.update(|mut settings| {
            if let Some(item) = settings.subscriptions.get(*id as usize) {
                match fs::remove_file(item.path.as_str()) {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("delete file error: {}", e);
                    }
                }
            }
            if let Some(item) = settings.subscriptions.get(*id as usize) {
                if settings.current_sub == item.path {
                    settings.current_sub = "".to_string();
                }
                settings.subscriptions.remove(*id as usize);
            }
        });
        match result {
            Ok(_) => vec![Primitive::Bool(true), Primitive::String("".to_string())],
            Err(e) => vec![Primitive::Bool(false), Primitive::String(e.to_string())]
        }
    }
}

pub fn set_sub(runtime: &Runtime) -> impl Fn(Vec<Primitive>) -> Vec<Primitive> {
    let runtime_clash = runtime.clash_state_clone();
    let runtime_setting = runtime.settings_clone();
    move |params: Vec<Primitive>| {
        if let Some(Primitive::String(path)) = params.get(0) {
            //更新到配置文件中
            runtime_setting.update(|mut x| x.current_sub = (*path).clone());
            //更新到当前内存中
            match runtime_clash.write() {
                Ok(mut x) => {
                    x.update_config_path(path);
                    log::info!("set profile path to {}", path);
                }
                Err(e) => {
                    log::error!("set_sub() failed to acquire clash write lock: {}", e);
                }
            }
        }
        return vec![];
    }
}

pub fn update_subs(runtime: &Runtime) -> impl AsyncCallable {
    let settings = runtime.settings.clone();
    move |_: Vec<Primitive>| {
        let settings = settings.clone();

        async move {
            let subs = settings.get().subscriptions;
            subscriptions::update_subs(subs).await;
            return vec![];
        }
    }
}

pub fn create_debug_log(_: Vec<Primitive>) -> Vec<Primitive> {
    let running_status = format!("Clash status : {}\n", utils::is_clash_running());
    let tomoon_config = match fs::read_to_string(utils::get_settings_path().unwrap()) {
        Ok(x) => x,
        Err(e) => {
            format!("can not get Tomoon config, error message: {} \n", e)
        }
    };
    let tomoon_log = match fs::read_to_string("/tmp/tomoon.log") {
        Ok(x) => x,
        Err(e) => {
            format!("can not get Tomoon log, error message: {} \n", e)
        }
    };
    let clash_log = match fs::read_to_string("/tmp/tomoon.clash.log") {
        Ok(x) => x,
        Err(e) => {
            format!("can not get Clash log, error message: {} \n", e)
        }
    };

    let log = format!(
        "
    {}\n
    ToMoon config:\n
    {}\n
    ToMoon log:\n
    {}\n
    Clash log:\n
    {}\n
    ",
        running_status, tomoon_config, tomoon_log, clash_log,
    );
    fs::write("/tmp/tomoon.debug.log", log).unwrap();
    return vec![true.into()];
}
