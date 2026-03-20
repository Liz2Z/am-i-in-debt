#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use am_i_in_debt::{
    get_current_selected_provider,
    logger::init_logging,
    login::run_login_script,
    provider::UsageInfo,
    providers::{get_provider_by_id, PROVIDERS},
    state::AppState,
    update_menu,
    menu::{rebuild_menu, update_menu_with_results},
    state::FetchResult,
};
use log::info;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    Manager,
};

fn main() {
    if let Err(e) = init_logging() {
        eprintln!("日志初始化失败: {}", e);
    }

    log::info!("应用启动");

    tauri::Builder::default()
        .manage(AppState::new())
        .setup(|app| {
            let tray_icon = include_bytes!("../icons/tray-icon.png");
            let icon = tauri::image::Image::from_bytes(&tray_icon.to_vec())
                .expect("Failed to load tray icon");

            let menu = Menu::with_items(
                app,
                &[
                    &MenuItem::with_id(app, "header", "Am I In Debt ?", true, None::<&str>)
                        .unwrap(),
                    &PredefinedMenuItem::separator(app).unwrap(),
                    &MenuItem::with_id(app, "status", "加载中...", false, None::<&str>).unwrap(),
                    &PredefinedMenuItem::separator(app).unwrap(),
                    &MenuItem::with_id(app, "quit", "退出", true, None::<&str>).unwrap(),
                ],
            )?;

            log::info!("开始创建托盘菜单");

            let tray = TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(handle_menu_event)
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { .. } = event {
                        let app = tray.app_handle().clone();
                        log::info!("检测到托盘点击，触发后台刷新");
                        tauri::async_runtime::spawn(async move {
                            let results = fetch_all_usage().await;
                            update_menu_with_results(&app, results);
                            check_exhausted_notification(&app);
                        });
                    }
                })
                .build(app)?;

            let state: tauri::State<AppState> = app.state();
            state.set_tray(tray);
            log::info!("托盘初始化完成");

            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                log::info!("执行首次额度拉取");
                let results = fetch_all_usage().await;
                update_menu_with_results(&app_handle, results);
                check_exhausted_notification(&app_handle);
            });

            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    let interval = {
                        let state: tauri::State<AppState> = app_handle.state();
                        state.get_refresh_interval()
                    };
                    log::info!("定时刷新：等待 {} 秒", interval);
                    tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
                    log::info!("执行定时刷新");
                    let results = fetch_all_usage().await;
                    update_menu_with_results(&app_handle, results);
                    check_exhausted_notification(&app_handle);
                }
            });

            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn fetch_all_usage() -> Vec<FetchResult> {
    let futures: Vec<_> = PROVIDERS
        .iter()
        .map(|provider| async move {
            log::info!("开始拉取 {} 额度信息", provider.display_name());
            let result = provider.fetch_usage(provider.cookie_path()).await;
            match &result {
                Ok(_) => log::info!("{} 额度拉取成功", provider.display_name()),
                Err(e) => log::warn!("{} 额度拉取失败: {}", provider.display_name(), e),
            }
            FetchResult {
                provider_id: provider.id(),
                result,
            }
        })
        .collect();

    futures::future::join_all(futures).await
}

fn check_exhausted_notification(app: &tauri::AppHandle) {
    let selected_provider_id = get_current_selected_provider();

    if let Some(provider_id) = selected_provider_id {
        let state: tauri::State<AppState> = app.state();

        state.with_usage(|usage_list| {
            if let Some(usage) = usage_list.iter().find(|u| u.provider_id() == provider_id) {
                if usage.is_token_exhausted() {
                    if state.should_notify_exhausted(&provider_id) {
                        if let Some(provider) = get_provider_by_id(&provider_id) {
                            send_notification(
                                &format!("{} Token 耗尽", provider.display_name()),
                                "Token 额度已用完，请等待重置或切换到其他平台",
                            );
                        }
                    }
                } else {
                    state.clear_exhausted_notification(&provider_id);
                }
            }
        });
    }
}

fn send_notification(title: &str, body: &str) {
    let _ = std::process::Command::new("osascript")
        .arg("-e")
        .arg(format!(
            r#"display notification "{}" with title "{}""#,
            body, title
        ))
        .spawn();
}

fn handle_menu_event(app: &tauri::AppHandle, event: tauri::menu::MenuEvent) {
    let event_id = event.id.as_ref();
    log::info!("收到菜单事件: {}", event_id);

    if let Some(provider_id) = event_id.strip_prefix("select-") {
        if let Some(provider) = get_provider_by_id(provider_id) {
            handle_select_provider(app, provider);
        }
        return;
    }

    if let Some(provider_id) = event_id
        .strip_prefix("login-")
        .or_else(|| event_id.strip_prefix("relogin-"))
    {
        if let Some(provider) = get_provider_by_id(provider_id) {
            handle_login(app, provider);
        }
        return;
    }

    if let Some(secs_str) = event_id.strip_prefix("interval-") {
        if let Ok(secs) = secs_str.parse::<u64>() {
            handle_interval(app, secs);
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

    if let Err(e) = am_i_in_debt::merge_settings(provider) {
        log::error!("切换到{}失败: {}", provider.display_name(), e);
        return;
    }

    info!("切换成功，重新构建菜单");
    rebuild_menu(app);
}

fn handle_login(app: &tauri::AppHandle, provider: &dyn am_i_in_debt::provider::Provider) {
    log::info!("触发登录流程: {}", provider.display_name());
    let provider_id = provider.id().to_string();
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let provider_ref = get_provider_by_id(&provider_id).unwrap();
        log::info!("开始执行 {} 登录脚本", provider_ref.display_name());

        if let Err(e) = run_login_script(&app_handle, provider_ref).await {
            log::error!("登录{}失败: {}", provider_ref.display_name(), e);
            return;
        }

        log::info!(
            "{} 登录脚本执行完成，等待 Cookie 写入",
            provider_ref.display_name()
        );
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        match provider_ref.fetch_usage(provider_ref.cookie_path()).await {
            Ok(info) => {
                log::info!("{} 登录后额度刷新成功", provider_ref.display_name());
                let state: tauri::State<AppState> = app_handle.state();
                state.with_usage(|usage_list| {
                    let mut new_list: Vec<Box<dyn UsageInfo>> = usage_list
                        .iter()
                        .filter(|u| u.provider_id() != provider_ref.id())
                        .map(|u| u.clone_boxed())
                        .collect();
                    new_list.push(info);
                    update_menu(&app_handle, new_list);
                });
            }
            Err(e) => {
                log::error!("{} 登录后额度刷新失败: {}", provider_ref.display_name(), e);
            }
        }
    });
}

fn handle_refresh(app: &tauri::AppHandle) {
    log::info!("手动刷新触发");
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let results = fetch_all_usage().await;
        update_menu_with_results(&app_handle, results);
        check_exhausted_notification(&app_handle);
    });
}

fn handle_interval(app: &tauri::AppHandle, secs: u64) {
    log::info!("切换刷新间隔为 {} 秒", secs);
    let state: tauri::State<AppState> = app.state();
    state.set_refresh_interval(secs);

    // 更新菜单中间隔选中状态
    use am_i_in_debt::menu::update_menu_in_place;
    update_menu_in_place(app);
}
