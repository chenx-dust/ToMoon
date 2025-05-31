use actix_web::{web, HttpResponse, Result};

use super::{ok, SingleParam};

use crate::{
    clash::{controller::EnhancedMode, runtime::Runtime}
};

macro_rules! set_setting_func {
    ($field:ident, $field_type:ty) => {
        pub async fn $field(
            state: web::Data<Runtime>,
            params: web::Form<SingleParam<$field_type>>,
        ) -> Result<HttpResponse> {
            state.settings.update(|mut x| x.$field = params.param.clone())?;
            ok()
        }
    };
}

set_setting_func!(skip_proxy, bool);
set_setting_func!(override_dns, bool);
set_setting_func!(allow_remote_access, bool);
set_setting_func!(enhanced_mode, EnhancedMode);
set_setting_func!(dashboard, String);
