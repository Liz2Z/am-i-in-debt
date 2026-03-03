#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use am_i_in_debt::{
    login::run_login_script,
    merge_settings,
    provider::UsageInfo,
    providers::{PROVIDERS, get_provider_by_id},
    state::AppState,
    update_menu,
};
use log::info;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Manager,
};

fn main() {
    #[cfg(debug_assertions)]
    {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
            .init();
    }
    
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
    for provider in PROVIDERS.iter() {
        if let Some(info) = provider.fetch_usage(provider.cookie_path()).await.ok() {
            usage_list.push(info);
        }
    }
    usage_list
}

fn handle_menu_event(app: &tauri::AppHandle, event: tauri::menu::MenuEvent) {
    let event_id = event.id.as_ref();

    if let Some(provider_id) = event_id.strip_prefix("select-") {
        if let Some(provider) = get_provider_by_id(provider_id) {
            handle_select_provider(app, provider);
        }
        return;
    }

    if let Some(provider_id) = event_id.strip_prefix("login-").or_else(|| event_id.strip_prefix("relogin-")) {
        if let Some(provider) = get_provider_by_id(provider_id) {
            handle_login(app, provider);
        }
        return;
    }

    match event_id {
        "refresh" => {
            handle_refresh(app);
        }
        "quit" => {
            std::process::exit(0);
        }
        _ => {}
    }
}

fn handle_select_provider(app: &tauri::AppHandle, provider: &dyn am_i_in_debt::provider::Provider) {
    info!("选择 {} Coding Plan", provider.display_name());
    
    if let Err(e) = merge_settings(provider) {
        log::error!("切换到{}失败: {}", provider.display_name(), e);
        return;
    }
    
    info!("切换成功，重新构建菜单");
    
    let state: tauri::State<AppState> = app.state();
    let usage_list = state.get_usage();
    update_menu(app, &usage_list);
}

fn handle_login(app: &tauri::AppHandle, provider: &dyn am_i_in_debt::provider::Provider) {
    let provider_id = provider.id().to_string();
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let provider_ref = get_provider_by_id(&provider_id).unwrap();
        
        if let Err(e) = run_login_script(provider_ref) {
            log::error!("登录{}失败: {}", provider_ref.display_name(), e);
            return;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        if let Some(info) = provider_ref.fetch_usage(provider_ref.cookie_path()).await.ok() {
            let state: tauri::State<AppState> = app_handle.state();
            let mut usage_list = state.get_usage();
            
            usage_list.retain(|u| u.provider_id() != provider_ref.id());
            
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
