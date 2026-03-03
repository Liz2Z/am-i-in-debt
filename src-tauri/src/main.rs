#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use am_i_in_debt::{
    api::fetch_usage_for_plan,
    login::run_login_script,
    models::CodingPlan,
    state::AppState,
    update_menu,
    UsageInfo,
};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Manager,
};

fn main() {
    tauri::Builder::default()
        .manage(AppState::new())
        .setup(|app| {
            let tray_icon = include_bytes!("../icons/tray-icon.png");
            let icon = tauri::image::Image::from_bytes(&tray_icon.to_vec())
                .expect("Failed to load tray icon");

            let menu = Menu::with_items(
                app,
                &[
                    &MenuItem::with_id(app, "header", "Am I In Debt ?", true, None::<&str>).unwrap(),
                    &PredefinedMenuItem::separator(app).unwrap(),
                    &MenuItem::with_id(app, "status", "加载中...", false, None::<&str>).unwrap(),
                    &PredefinedMenuItem::separator(app).unwrap(),
                    &MenuItem::with_id(app, "quit", "退出", true, None::<&str>).unwrap(),
                ],
            )?;

            let tray = TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(handle_menu_event)
                .build(app)?;

            let state: tauri::State<AppState> = app.state();
            state.set_tray(tray);

            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let usage_list = fetch_all_usage().await;
                update_menu(&app_handle, &usage_list);
            });

            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                    let usage_list = fetch_all_usage().await;
                    update_menu(&app_handle, &usage_list);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn fetch_all_usage() -> Vec<UsageInfo> {
    let mut usage_list = Vec::new();

    if let Some(info) = fetch_usage_for_plan(CodingPlan::Zhipu).await {
        usage_list.push(info);
    }

    if let Some(info) = fetch_usage_for_plan(CodingPlan::Kimi).await {
        usage_list.push(info);
    }

    usage_list
}

fn handle_menu_event(app: &tauri::AppHandle, event: tauri::menu::MenuEvent) {
    let event_id = event.id.as_ref();

    match event_id {
        "login-zhipu" | "relogin-zhipu" => {
            handle_login(app, CodingPlan::Zhipu);
        }
        "login-kimi" | "relogin-kimi" => {
            handle_login(app, CodingPlan::Kimi);
        }
        "refresh" => {
            handle_refresh(app);
        }
        "quit" => {
            std::process::exit(0);
        }
        _ => {}
    }
}

fn handle_login(app: &tauri::AppHandle, plan: CodingPlan) {
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = run_login_script(plan) {
            eprintln!("登录{}失败: {}", plan.name(), e);
            return;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        if let Some(info) = fetch_usage_for_plan(plan).await {
            let state: tauri::State<AppState> = app_handle.state();
            let mut usage_list = state.get_usage();
            
            match plan {
                CodingPlan::Zhipu => usage_list.retain(|u| !u.is_zhipu()),
                CodingPlan::Kimi => usage_list.retain(|u| !u.is_kimi()),
            }
            
            usage_list.push(info);
            update_menu(&app_handle, &usage_list);
        }
    });
}

fn handle_refresh(app: &tauri::AppHandle) {
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let usage_list = fetch_all_usage().await;
        update_menu(&app_handle, &usage_list);
    });
}
