use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::sync::{Arc, RwLock, RwLockWriteGuard};


use crate::clash::controller::EnhancedMode;

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    #[serde(default = "default_backend_port")]
    pub backend_port: u16,
    #[serde(default = "default_external_port")]
    pub external_port: u16,
    #[serde(default = "default_skip_proxy")]
    pub skip_proxy: bool,
    #[serde(default = "default_override_dns")]
    pub override_dns: bool,
    #[serde(default = "default_enhanced_mode")]
    pub enhanced_mode: EnhancedMode,
    #[serde(default = "default_current_sub")]
    pub current_sub: String,
    #[serde(default = "default_subscriptions")]
    pub subscriptions: Vec<Subscription>,
    #[serde(default = "default_allow_remote_access")]
    pub allow_remote_access: bool,
    #[serde(default = "default_dashboard")]
    pub dashboard: String,
    #[serde(default = "default_secret")]
    pub secret: String,
}

fn default_backend_port() -> u16 {
    55555
}

fn default_external_port() -> u16 {
    55556
}

fn default_skip_proxy() -> bool {
    true
}

fn default_override_dns() -> bool {
    true
}

fn default_allow_remote_access() -> bool {
    false
}

fn default_enhanced_mode() -> EnhancedMode {
    EnhancedMode::FakeIp
}

fn default_dashboard() -> String {
    "yacd-meta".to_string()
}

fn default_secret() -> String {
    "".to_string()
}

fn default_current_sub() -> String {
    "".to_string()
}

fn default_subscriptions() -> Vec<Subscription> {
    Vec::new()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Subscription {
    pub path: String,
    pub url: String,
}

#[derive(Debug)]
pub enum SettingsError {
    Serde(serde_json::Error),
    Io(std::io::Error),
}

impl Display for SettingsError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Serde(e) => (e as &dyn Display).fmt(f),
            Self::Io(e) => (e as &dyn Display).fmt(f),
        }
    }
}

impl Subscription {
    pub fn new(path: String, url: String) -> Self {
        Self {
            path: path,
            url: url,
        }
    }
}

impl Settings {
    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), SettingsError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(SettingsError::Io)?;
        }
        let mut file = std::fs::File::create(path).map_err(SettingsError::Io)?;
        serde_json::to_writer_pretty(&mut file, &self).map_err(SettingsError::Serde)
    }

    pub fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Settings, SettingsError> {
        let mut file = std::fs::File::open(path).map_err(SettingsError::Io)?;
        serde_json::from_reader(&mut file).map_err(SettingsError::Serde)
    }
}

impl Default for Settings {
    fn default() -> Self {
        serde_json::from_str("{}").unwrap()
    }
}

#[derive(Clone)]
pub struct SettingsInstance {
    settings: Arc<RwLock<Settings>>,
    path: std::path::PathBuf,
}

impl SettingsInstance {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Self {
        Self {
            settings: Arc::new(RwLock::new(Settings::default())),
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self, SettingsError> {
        if path.as_ref().exists() {
            let settings = Arc::new(RwLock::new(Settings::open(path.as_ref())?));
            Ok(Self {
                settings,
                path: path.as_ref().to_path_buf(),
            })
        } else {
            let instance = Self::new(path);
            instance.save()?;
            Ok(instance)
        }
    }

    pub fn save(&self) -> Result<(), SettingsError> {
        self.settings.write().unwrap().save(&self.path)?;
        Ok(())
    }

    pub fn get(&self) -> Settings {
        self.settings.read().unwrap().clone()
    }

    pub fn update<T>(&self, function: impl Fn (RwLockWriteGuard<'_, Settings>) -> T) -> Result<T, SettingsError> {
        let result = function(self.settings.write().unwrap());
        self.save()?;
        Ok(result)
    }
}
