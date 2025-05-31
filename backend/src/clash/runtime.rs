use std::sync::{Arc, RwLock};

use crate::utils;
use crate::settings::SettingsInstance;

use super::controller::Controller;

#[derive(Clone)]
pub struct Runtime {
    pub settings: SettingsInstance,
    pub controller: Arc<RwLock<Controller>>,
}


impl Runtime {
    pub fn new() -> Self {
        let settings_path = utils::get_settings_path().unwrap();
        let clash = Controller::default();
        Self {
            settings: SettingsInstance::open(settings_path).unwrap(),
            controller: Arc::new(RwLock::new(clash)),
        }
    }
}
