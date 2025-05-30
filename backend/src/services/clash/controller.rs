use std::fmt::Display;
use std::process::{Child, Command};

use std::{error, fs};

use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};

use crate::utils;

use serde_json::json;

pub struct Controller {
    pub path: std::path::PathBuf,
    pub config: std::path::PathBuf,
    pub instance: Option<Child>,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum EnhancedMode {
    RedirHost,
    FakeIp,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ClashErrorKind {
    ConfigFormatError,
    ConfigNotFound,
    NetworkError,
    InnerError,
    Default,
}

#[derive(Debug)]
pub struct ClashError {
    pub message: String,
    pub error_kind: ClashErrorKind,
}

impl error::Error for ClashError {}

impl Display for ClashError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error Kind: {:?}, Error Message: {})",
            self.error_kind, self.message
        )
    }
}

impl ClashError {
    pub fn new() -> Self {
        Self {
            message: "".to_string(),
            error_kind: ClashErrorKind::Default,
        }
    }
}

impl Default for Controller {
    fn default() -> Self {
        Self {
            path: utils::get_current_working_dir().unwrap().join("bin/core/clash"),
            config: utils::get_decky_data_dir()
                .unwrap()
                .join("config.yaml"),
            instance: None,
        }
    }
}

impl Controller {
    pub fn run(
        &mut self,
        config_path: &String,
        skip_proxy: bool,
        override_dns: bool,
        allow_remote_access: bool,
        enhanced_mode: EnhancedMode,
        dashboard: String,
    ) -> Result<(), ClashError> {
        // decky 插件数据目录
        let decky_data_dir = utils::get_decky_data_dir().unwrap();
        let clash_dir = utils::get_current_working_dir().unwrap().join("bin/core");

        // 检查 decky_data_dir 是否存在，不存在则创建
        if !decky_data_dir.exists() {
            fs::create_dir_all(&decky_data_dir).unwrap();
        }

        self.update_config_path(config_path);
        // 修改配置文件为推荐配置
        match self.change_config(
            skip_proxy,
            override_dns,
            allow_remote_access,
            enhanced_mode,
            dashboard,
        ) {
            Ok(_) => (),
            Err(e) => {
                return Err(ClashError {
                    message: e.to_string(),
                    error_kind: ClashErrorKind::ConfigFormatError,
                });
            }
        }

        //log::info!("Pre-setting network");
        //TODO: 未修改的 unwarp
        let run_config = self.get_running_config().unwrap();
        let outputs = fs::File::create("/tmp/tomoon.clash.log").unwrap();
        let errors = outputs.try_clone().unwrap();

        log::info!("Starting Clash...");

        let clash = Command::new(self.path.clone())
            .arg("-d")
            .arg(clash_dir)
            .arg("-f")
            .arg(run_config)
            .stdout(outputs)
            .stderr(errors)
            .spawn();
        let clash: Result<Child, ClashError> = match clash {
            Ok(x) => {
                Ok(x)
            },
            Err(e) => {
                log::error!("run Clash failed: {}", e);
                //TODO: 开启 Clash 的错误处理
                return Err(ClashError::new());
            }
        };
        self.instance = Some(clash.unwrap());
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), Box<dyn error::Error>> {
        let instance = self.instance.as_mut();
        match instance {
            Some(x) => {
                x.kill()?;
                x.wait()?;

                //直接重置网络
                utils::reset_system_network()?;
            }
            None => {
                //Not launch Clash yet...
                log::error!("Error occurred while disabling Clash: Not launch Clash yet");
            }
        };
        Ok(())
    }

    pub fn update_config_path(&mut self, path: &String) {
        self.config = std::path::PathBuf::from((*path).clone());
    }

    pub fn get_running_config(&self) -> std::io::Result<std::path::PathBuf> {
        let decky_data_dir = utils::get_decky_data_dir().unwrap();
        let run_config = decky_data_dir.join("running_config.yaml");
        Ok(run_config)
    }

    pub async fn reload_config(&self) -> Result<(), ClashError> {
        let run_config = self.get_running_config().unwrap();
        log::info!("Reloading Clash config, config: {}", run_config.display());

        let url = "http://127.0.0.1:9090/configs?reload=true";
        let body = json!({
            "path": run_config,
            "payload": ""
        });

        let body_str = serde_json::to_string(&body).unwrap();

        let res = match minreq::put(url)
            .with_header("Content-Type", "application/json")
            .with_body(body_str)
            .send()
        {
            Ok(x) => x,
            Err(e) => {
                log::error!("Failed to restart Clash core: {}", e);
                return Err(ClashError {
                    message: e.to_string(),
                    error_kind: ClashErrorKind::InnerError,
                });
            }
        };

        if res.status_code == 200 || res.status_code == 204 {
            log::info!("Clash config reloaded successfully");
        } else {
            log::error!(
                "Failed to reload Clash config, status_code {}",
                res.status_code
            );
        }

        Ok(())
    }

    pub async fn restart_core(&self) -> Result<(), ClashError> {
        log::info!("Restarting Clash core...");

        let url = "http://127.0.0.1:9090/restart";
        let body = json!({
            "payload": ""
        });
        let body_str = serde_json::to_string(&body).unwrap();

        let res = match minreq::post(url)
            .with_header("Content-Type", "application/json")
            .with_body(body_str)
            .send()
        {
            Ok(x) => x,
            Err(e) => {
                log::error!("Failed to restart Clash core: {}", e);
                return Err(ClashError {
                    message: e.to_string(),
                    error_kind: ClashErrorKind::InnerError,
                });
            }
        };

        if res.status_code == 200 {
            log::info!("Clash restart successfully");
        } else {
            let data = res.as_str().unwrap();
            log::error!("Failed to restart Clash core: {}", data);
        }

        Ok(())
    }

    pub fn change_config(
        &self,
        skip_proxy: bool,
        override_dns: bool,
        allow_remote_access: bool,
        enhanced_mode: EnhancedMode,
        dashboard: String,
    ) -> Result<(), Box<dyn error::Error>> {
        let path = self.config.clone();
        log::info!("change_config path: {:?}", path);

        let config = fs::read_to_string(path)?;
        let mut yaml: serde_yaml::Value = serde_yaml::from_str(config.as_str())?;
        let yaml = yaml.as_mapping_mut().unwrap();

        log::info!("Changing Clash config...");

        let external_ip = if allow_remote_access {
            "0.0.0.0"
        } else {
            "127.0.0.1"
        };

        //修改 WebUI
        match yaml.get_mut("external-controller") {
            Some(x) => {
                *x = Value::String(String::from(format!("{}:9090", external_ip)));
            }
            None => {
                yaml.insert(
                    Value::String(String::from("external-controller")),
                    Value::String(String::from(format!("{}:9090", external_ip))),
                );
            }
        }

        //修改 test.steampowered.com
        //这个域名用于 Steam Deck 网络连接验证，可以直连
        if let Some(x) = yaml.get_mut("rules") {
            let rules = x.as_sequence_mut().unwrap();
            rules.insert(
                0,
                Value::String(String::from("DOMAIN,test.steampowered.com,DIRECT")),
            );

            if skip_proxy {
                rules.insert(
                    0,
                    Value::String(String::from("DOMAIN-SUFFIX,cm.steampowered.com,DIRECT")),
                );
                rules.insert(
                    0,
                    Value::String(String::from("DOMAIN-SUFFIX,steamserver.net,DIRECT")),
                );
            }
        }

        let webui_dir = utils::get_current_working_dir()?.join("bin/core/web");

        match yaml.get_mut("external-ui") {
            Some(x) => {
                //TODO: 修改 Web UI 的路径
                *x = Value::String(String::from(webui_dir.to_str().unwrap()));
            }
            None => {
                yaml.insert(
                    Value::String(String::from("external-ui")),
                    Value::String(String::from(webui_dir.to_str().unwrap())),
                );
            }
        }

        //  修改 dashboard 名称
        match yaml.get_mut("external-ui-name") {
            Some(x) => {
                *x = Value::String(String::from(dashboard));
            }
            None => {
                yaml.insert(
                    Value::String(String::from("external-ui-name")),
                    Value::String(String::from(dashboard)),
                );
            }
        }

        //修改 TUN 和 DNS 配置

        let tun_config = "
        enable: true
        stack: system
        auto-route: true
        auto-detect-interface: true
        dns-hijack:
            - any:53
        ";

        //部分配置来自 https://www.xkww3n.cyou/2022/02/08/use-clash-dns-anti-dns-hijacking/

        let dns_config_fakeip = "
        enable: true
        listen: 127.0.0.1:8853
        default-nameserver:
            - 223.5.5.5
            - 8.8.4.4
        ipv6: false
        enhanced-mode: fake-ip
        nameserver:
            - 119.29.29.29
            - 223.5.5.5
            - tls://223.5.5.5:853
            - tls://223.6.6.6:853
        fallback:
            - https://1.0.0.1/dns-query
            - https://public.dns.iij.jp/dns-query
            - tls://8.8.4.4:853
        fallback-filter:
            geoip: false
            ipcidr:
            - 240.0.0.0/4
            - 0.0.0.0/32
            - 127.0.0.1/32
        fake-ip-filter:
            - \"*.lan\"
            - \"*.localdomain\"
            - \"*.localhost\"
            - \"*.local\"
            - \"*.home.arpa\"
            - stun.*.*
            - stun.*.*.*
            - +.stun.*.*
            - +.stun.*.*.*
            - +.stun.*.*.*.*
        ";

        let dns_config_redir_host = "
        enable: true
        ipv6: false
        listen: 127.0.0.1:8853
        default-nameserver:
            - 223.5.5.5
            - 8.8.4.4
        enhanced-mode: redir-host
        nameserver:
            - 119.29.29.29
            - 223.5.5.5
            - tls://223.5.5.5:853
            - tls://223.6.6.6:853
        fallback:
            - https://1.0.0.1/dns-query
            - https://public.dns.iij.jp/dns-query
            - tls://8.8.4.4:853
        fallback-filter:
            geoip: false
            ipcidr:
            - 240.0.0.0/4
            - 0.0.0.0/32
            - 127.0.0.1/32
        ";

        let profile_config = "
        store-selected: true
        store-fake-ip: false
        ";

        let insert_config = |yaml: &mut Mapping, config: &str, key: &str| {
            let inner_config: Value = serde_yaml::from_str(config).unwrap();
            yaml.insert(Value::String(String::from(key)), inner_config);
        };

        //开启 tun 模式
        match yaml.get("tun") {
            Some(_) => {
                yaml.remove("tun").unwrap();
                insert_config(yaml, tun_config, "tun");
            }
            None => {
                insert_config(yaml, tun_config, "tun");
            }
        }

        match yaml.get("dns") {
            Some(_) => {
                //删除 DNS 配置
                if override_dns {
                    log::info!("EnhancedMode: {:?}", enhanced_mode);
                    yaml.remove("dns").unwrap();
                    match enhanced_mode {
                        EnhancedMode::FakeIp => {
                            insert_config(yaml, dns_config_fakeip, "dns");
                        }
                        EnhancedMode::RedirHost => {
                            insert_config(yaml, dns_config_redir_host, "dns");
                        }
                    }
                }
            }
            None => {
                insert_config(yaml, dns_config_fakeip, "dns");
            }
        }

        // // 如果设置了 secret， 更改 secret 为 "tomoon"
        // let secret_config = "tomoon";
        // match yaml.get("secret") {
        //     Some(_) => {
        //         yaml.remove("secret").unwrap();
        //         insert_config(yaml, secret_config, "secret");
        //     }
        //     None => {
        //         insert_config(yaml, secret_config, "secret");
        //     }
        // }

        // 保存上次的配置
        match yaml.get("profile") {
            Some(_) => {
                yaml.remove("profile").unwrap();
                insert_config(yaml, profile_config, "profile");
            }
            None => {
                insert_config(yaml, profile_config, "profile");
            }
        }

        let run_config = self.get_running_config()?;

        let yaml_str = serde_yaml::to_string(&yaml)?;

        match fs::write(run_config, yaml_str) {
            Ok(_) => {
                log::info!("Clash config changed successfully");
            }
            Err(e) => {
                log::error!("Error occurred while changing Clash config: {}", e);
            }
        }

        log::info!("Clash config changed successfully");
        Ok(())
    }

    pub fn get_running_secret(&self) -> Result<String, Box<dyn error::Error>> {
        let path = self.get_running_config()?;
        let content = std::fs::read_to_string(path)?;
        let yaml: serde_yaml::Value = serde_yaml::from_str(&content)?;
        
        match yaml.get("secret") {
            Some(secret) => {
                Ok(secret.as_str().unwrap_or("").to_string())
            }
            None => Ok("".to_string()),
        }
    }
}
