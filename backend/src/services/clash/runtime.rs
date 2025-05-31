use std::sync::{Arc, RwLock};

use crate::utils;
use crate::settings::{Settings, SettingsInstance};

use super::controller::Controller;

pub struct Runtime {
    settings: SettingsInstance,
    clash_state: Arc<RwLock<Controller>>,
    downlaod_status: Arc<RwLock<DownloadStatus>>,
    update_status: Arc<RwLock<DownloadStatus>>,
    running_status: Arc<RwLock<RunningStatus>>,
}

#[derive(Debug)]
pub enum RunningStatus {
    Loading,
    Failed,
    Success,
    None,
}

#[derive(Debug)]
pub enum DownloadStatus {
    Downloading,
    Failed,
    Success,
    None,
}

impl std::fmt::Display for DownloadStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

impl std::fmt::Display for RunningStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

impl Runtime {
    pub fn new() -> Self {
        let settings_path = utils::get_settings_path().unwrap();
        //TODO: Clash 路径
        let clash = Controller::default();
        let download_status = DownloadStatus::None;
        let update_status = DownloadStatus::None;
        let running_status = RunningStatus::None;
        Self {
            settings: SettingsInstance::open(settings_path).unwrap(),
            clash_state: Arc::new(RwLock::new(clash)),
            downlaod_status: Arc::new(RwLock::new(download_status)),
            update_status: Arc::new(RwLock::new(update_status)),
            running_status: Arc::new(RwLock::new(running_status)),
        }
    }

    pub(crate) fn settings_clone(&self) -> SettingsInstance {
        self.settings.clone()
    }

    pub fn clash_state_clone(&self) -> Arc<RwLock<Controller>> {
        self.clash_state.clone()
    }

    pub fn downlaod_status_clone(&self) -> Arc<RwLock<DownloadStatus>> {
        self.downlaod_status.clone()
    }

    pub fn update_status_clone(&self) -> Arc<RwLock<DownloadStatus>> {
        self.update_status.clone()
    }

    pub fn running_status_clone(&self) -> Arc<RwLock<RunningStatus>> {
        self.running_status.clone()
    }

}
