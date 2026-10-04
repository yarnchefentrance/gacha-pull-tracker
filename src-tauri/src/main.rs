// Wishboard - gacha pull tracker and pity counter for Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod import;
mod models;
mod odds;
mod pity;
mod planner;
mod sample;
mod state;
mod uigf;

use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .manage(state::AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::list_games,
            commands::get_overview,
            commands::get_history,
            commands::get_odds,
            commands::planner_preset,
            commands::plan_savings,
            commands::detect_import,
            commands::start_import,
            commands::export_uigf,
            commands::import_uigf,
        ])
        .setup(|app| {
            let show = MenuItem::with_id(app, "show", "Open Wishboard", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;
            let mut tray = TrayIconBuilder::with_id("main").tooltip("Wishboard").menu(&menu).on_menu_event(|app, ev| match ev.id().as_ref() {
                "show" => {
                    if let Some(w) = app.get_webview_window("main") {
                        let _ = w.show();
                        let _ = w.set_focus();
                    }
                }
                "quit" => app.exit(0),
                _ => {}
            });
            if let Some(icon) = app.default_window_icon() {
                tray = tray.icon(icon.clone());
            }
            tray.build(app)?;
            log::info!("Wishboard started, data dir {}", state::data_dir().display());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Wishboard");
}
