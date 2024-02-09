// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod log;
pub mod model;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let _ = model::Model::init(app);
            log::setup_logger(app, log::LoggerOutput::Stdout, tracing::Level::DEBUG);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            model::logic::predict,
            model::logic::load_llm_model,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
