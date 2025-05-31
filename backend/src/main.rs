mod api;
mod clash;
mod utils;
mod settings;
mod subscriptions;
mod test;

use actix_cors::Cors;
use actix_files as fs;
use actix_web::{http::Method, middleware, web, App, HttpServer};
use simplelog::{ColorChoice, CombinedLogger, LevelFilter, TermLogger, TerminalMode, WriteLogger};

use crate::clash::runtime::Runtime;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
pub const PACKAGE_NAME: &'static str = env!("CARGO_PKG_NAME");

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    if cfg!(debug_assertions) {
        CombinedLogger::init(
            vec![
                TermLogger::new(
                    LevelFilter::Debug,
                    Default::default(),
                    TerminalMode::Mixed,
                    ColorChoice::Auto,
                ),
                WriteLogger::new(
                    LevelFilter::Debug,
                    Default::default(),
                    std::fs::File::create(utils::get_decky_logs_dir().unwrap().join("tomoon.json")).unwrap(),
                ),
            ],
        ).unwrap();
    } else {
        WriteLogger::init(
            LevelFilter::Info,
            Default::default(),
            std::fs::File::create(utils::get_decky_logs_dir().unwrap().join("tomoon.json")).unwrap(),
        )
        .unwrap();
    }

    log::info!("Starting back-end ({} v{})", PACKAGE_NAME, VERSION);
    log::info!("{}", std::env::current_dir().unwrap().to_str().unwrap());
    println!("Starting back-end ({} v{})", PACKAGE_NAME, VERSION);

    let runtime = Runtime::new();
    let runtime_cp = runtime.clone();
    let backend_port = runtime.settings.get().backend_port;
    let external_port = runtime.settings.get().external_port;

    tokio::spawn(
        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(runtime.clone()))
                .wrap(middleware::Logger::default())
                .wrap(Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec![Method::GET, Method::POST])
                    .allow_any_header())
                // 公共接口
                .service(
                    web::resource("/download_sub")
                    .route(web::post().to(api::controller::download_sub)))
                // web
                .service(
                    fs::Files::new("/", "./web")
                        .index_file("index.html")
                        .show_files_listing(),
                )
        })
        .bind(("0.0.0.0", external_port))?
        .bind(("[::]", external_port))?
        .workers(1)
        .run()
    );

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(runtime_cp.clone()))
            .wrap(middleware::Logger::default())
            .wrap(Cors::permissive())
            // 功能接口
            .service(
                web::resource("/get_ip_address")
                .route(web::get().to(api::controller::get_local_web_address)))
            .service(
                web::resource("/reload_clash_config")
                    .route(web::get().to(api::controller::reload_clash_config)))
            .service(
                web::resource("/restart_clash")
                .route(web::get().to(api::controller::restart_clash)))
            .service(
                web::resource("/download_sub")
                .route(web::post().to(api::controller::download_sub)))
            // 设置值
            .service(
                web::resource("/skip_proxy")
                .route(web::post().to(api::settings::skip_proxy)))
            .service(
                web::resource("/override_dns")
                .route(web::post().to(api::settings::override_dns)))
            .service(
                web::resource("/enhanced_mode")
                .route(web::post().to(api::settings::enhanced_mode)))
            .service(
                web::resource("/allow_remote_access")
                    .route(web::post().to(api::settings::allow_remote_access)))
            .service(
                web::resource("/dashboard")
                .route(web::post().to(api::settings::dashboard)))
    })
    .bind(("localhost", backend_port))?
    .workers(1)
    .run()
    .await
}
