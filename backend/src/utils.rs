use std::process::Command;

use regex::Regex;

use sysinfo::{ProcessExt, System, SystemExt};

pub fn get_current_working_dir() -> std::io::Result<std::path::PathBuf> {
    std::env::current_dir()
}

pub fn check_yaml(str: &String) -> bool {
    if let Ok(x) = serde_yaml::from_str::<serde_yaml::Value>(str) {
        if let Some(v) = x.as_mapping() {
            if v.contains_key("rules") {
                return true;
            } else {
                return false;
            }
        } else {
            return false;
        }
    } else {
        return false;
    }
}

pub fn is_clash_running() -> bool {
    //关闭 systemd-resolved
    let mut sys = System::new_all();
    sys.refresh_all();
    for (_, process) in sys.processes() {
        if process.name() == "clash" {
            return true;
        }
    }
    return false;
}

pub fn get_file_path(url: String) -> Option<String> {
    let r = Regex::new(r"^file://").unwrap();
    if let Some(x) = r.find(url.clone().as_str()) {
        let file_path = url[x.end()..url.len()].to_string();
        return Some(file_path);
    };
    return None;
}

pub fn get_decky_data_dir() -> std::io::Result<std::path::PathBuf> {
    let data_dir = get_current_working_dir()?
        .parent().ok_or(std::io::ErrorKind::AddrNotAvailable)?
        .parent().ok_or(std::io::ErrorKind::AddrNotAvailable)?
        .join("data/tomoon");
    Ok(data_dir)
}

pub fn get_settings_path() -> std::io::Result<std::path::PathBuf> {
    let path = get_decky_data_dir()?.join("tomoon.json");
    Ok(path)
}

pub fn get_sub_dir() -> std::io::Result<std::path::PathBuf> {
    let path = get_decky_data_dir()?.join("subs");
    Ok(path)
}
