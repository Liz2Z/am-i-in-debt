use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    AppHandle, Manager, Wry,
};

use crate::provider::UsageInfo;
use crate::provider_switch::get_current_selected_provider;
use crate::providers::PROVIDERS;
use crate::state::{AppState, FetchResult, FetchStatus, MenuHandles};

/// 刷新间隔选项（秒）
const INTERVAL_OPTIONS: &[(u64, &str)] = &[
    (60, "1 分钟"),
    (300, "5 分钟"),
    (1800, "30 分钟"),
];

pub fn build_menu(
    app: &AppHandle,
    usage_list: &[Box<dyn UsageInfo>],
    update_time_suffix: &str,
) -> Menu<Wry> {
    let selected_provider = get_current_selected_provider();

    if usage_list.is_empty() {
        build_empty_menu(app, selected_provider)
    } else {
        build_usage_menu(app, usage_list, update_time_suffix, selected_provider)
    }
}

fn build_empty_menu(app: &AppHandle, selected_provider_id: Option<String>) -> Menu<Wry> {
    let header =
        MenuItem::with_id(app, "header", "Am I In Debt ?", true, None::<&str>).unwrap();
    let sep1 = PredefinedMenuItem::separator(app).unwrap();

    let mut items: Vec<Box<dyn tauri::menu::IsMenuItem<Wry>>> =
        vec![Box::new(header), Box::new(sep1)];

    for provider in PROVIDERS.iter() {
        let is_checked = selected_provider_id.as_deref() == Some(provider.id());
        let item = CheckMenuItem::with_id(
            app,
            format!("select-{}", provider.id()),
            format!("{} Coding Plan", provider.display_name()),
            true,
            is_checked,
            None::<&str>,
        )
        .unwrap();
        items.push(Box::new(item));
    }

    let sep2 = PredefinedMenuItem::separator(app).unwrap();
    items.push(Box::new(sep2));

    for provider in PROVIDERS.iter() {
        let item = MenuItem::with_id(
            app,
            format!("login-{}", provider.id()),
            format!("登录{} Coding Plan", provider.display_name()),
            true,
            None::<&str>,
        )
        .unwrap();
        items.push(Box::new(item));
    }

    let sep3 = PredefinedMenuItem::separator(app).unwrap();
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>).unwrap();
    items.push(Box::new(sep3));
    items.push(Box::new(quit));

    let items_refs: Vec<&dyn tauri::menu::IsMenuItem<Wry>> =
        items.iter().map(|item| item.as_ref()).collect();
    Menu::with_items(app, &items_refs).unwrap()
}

fn build_usage_menu(
    app: &AppHandle,
    usage_list: &[Box<dyn UsageInfo>],
    update_time_suffix: &str,
    selected_provider_id: Option<String>,
) -> Menu<Wry> {
    let state: tauri::State<AppState> = app.state();
    let current_interval = state.get_refresh_interval();

    let header =
        MenuItem::with_id(app, "header", "Am I In Debt ?", true, None::<&str>).unwrap();
    let sep1 = PredefinedMenuItem::separator(app).unwrap();

    let mut items: Vec<Box<dyn tauri::menu::IsMenuItem<Wry> + '_>> =
        vec![Box::new(header), Box::new(sep1)];

    let mut handles = MenuHandles::new();

    for provider in PROVIDERS.iter() {
        if let Some(usage) = usage_list.iter().find(|u| u.provider_id() == provider.id()) {
            let is_selected = selected_provider_id.as_deref() == Some(provider.id());
            let menu_items = usage.render_menu_items(app, is_selected, &mut handles);

            for item in menu_items {
                items.push(item);
            }

            // 检查是否有拉取失败状态
            if let Some(status) = state.get_fetch_status(provider.id()) {
                if let FetchStatus::HttpError(_) = status {
                    let warn_item = MenuItem::with_id(
                        app,
                        format!("warn-{}", provider.id()),
                        "⚠️ 上次更新失败，显示的是缓存数据",
                        false,
                        None::<&str>,
                    )
                    .unwrap();
                    handles
                        .items
                        .insert(format!("warn-{}", provider.id()), warn_item.clone());
                    items.push(Box::new(warn_item));
                }
            }
        } else {
            items.push(Box::new(
                MenuItem::with_id(
                    app,
                    format!("login-{}", provider.id()),
                    format!("登录{} Coding Plan", provider.display_name()),
                    true,
                    None::<&str>,
                )
                .unwrap(),
            ));
        }
    }

    items.push(Box::new(PredefinedMenuItem::separator(app).unwrap()));

    let refresh_item = MenuItem::with_id(
        app,
        "refresh",
        format!("刷新{}", update_time_suffix),
        true,
        None::<&str>,
    )
    .unwrap();
    handles
        .items
        .insert("refresh".to_string(), refresh_item.clone());
    items.push(Box::new(refresh_item));

    for provider in PROVIDERS.iter() {
        if usage_list.iter().any(|u| u.provider_id() == provider.id()) {
            items.push(Box::new(
                MenuItem::with_id(
                    app,
                    format!("relogin-{}", provider.id()),
                    format!("重新登录{}", provider.display_name()),
                    true,
                    None::<&str>,
                )
                .unwrap(),
            ));
        }
    }

    items.push(Box::new(PredefinedMenuItem::separator(app).unwrap()));

    // 刷新间隔选项
    let interval_title = MenuItem::with_id(
        app,
        "interval-title",
        "刷新间隔",
        false,
        None::<&str>,
    )
    .unwrap();
    items.push(Box::new(interval_title));

    for (secs, label) in INTERVAL_OPTIONS.iter() {
        let is_current = *secs == current_interval;
        let check = CheckMenuItem::with_id(
            app,
            format!("interval-{}", secs),
            format!("  {}", label),
            true,
            is_current,
            None::<&str>,
        )
        .unwrap();
        handles
            .checks
            .insert(format!("interval-{}", secs), check.clone());
        items.push(Box::new(check));
    }

    items.push(Box::new(PredefinedMenuItem::separator(app).unwrap()));
    items.push(Box::new(
        MenuItem::with_id(app, "quit", "退出", true, None::<&str>).unwrap(),
    ));

    let items_refs: Vec<&dyn tauri::menu::IsMenuItem<Wry>> =
        items.iter().map(|item| item.as_ref()).collect();
    let menu = Menu::with_items(app, &items_refs).unwrap();

    state.store_menu_handles(handles);

    menu
}



/// 原地更新菜单项内容（不重建菜单）
pub fn update_menu_in_place(app: &AppHandle) {
    let state: tauri::State<AppState> = app.state();

    state.update_time();
    let update_time_suffix = state.get_update_time_suffix();

    let menu_handles = state.menu_handles.lock().unwrap();
    if let Some(handles) = menu_handles.as_ref() {
        // 更新数据项
        state.with_usage(|usage_list| {
            for usage in usage_list {
                for (item_id, new_text) in usage.menu_item_updates() {
                    if let Some(item) = handles.items.get(&item_id) {
                        let _ = item.set_text(new_text);
                    }
                }

                // 更新/移除警告项
                let warn_id = format!("warn-{}", usage.provider_id());
                if let Some(warn_item) = handles.items.get(&warn_id) {
                    match state.get_fetch_status(usage.provider_id()) {
                        Some(FetchStatus::HttpError(_)) => {
                            let _ = warn_item.set_text("⚠️ 上次更新失败，显示的是缓存数据");
                        }
                        _ => {
                            let _ = warn_item.set_text("");
                            let _ = warn_item.set_enabled(false);
                        }
                    }
                }
            }
        });

        // 更新刷新按钮
        if let Some(refresh_item) = handles.items.get("refresh") {
            let _ = refresh_item.set_text(format!("刷新{}", update_time_suffix));
        }

        // 更新间隔选中状态
        let current_interval = state.get_refresh_interval();
        for (secs, _) in INTERVAL_OPTIONS.iter() {
            let check_id = format!("interval-{}", secs);
            if let Some(check) = handles.checks.get(&check_id) {
                let _ = check.set_checked(*secs == current_interval);
            }
        }
    }
}

/// 全量重建菜单（用于首次构建或结构性变化如登录/登出）
pub fn rebuild_menu(app: &AppHandle) {
    let state: tauri::State<AppState> = app.state();
    state.update_time();

    let update_time_suffix = state.get_update_time_suffix();

    state.with_usage(|list| {
        let menu = build_menu(app, list, &update_time_suffix);
        if let Some(tray) = state.get_tray() {
            let _ = tray.set_menu(Some(menu));
        }
    });
}

/// 合并结果并更新菜单 — 自动选择原地更新或全量重建
pub fn update_menu_with_results(app: &AppHandle, results: Vec<FetchResult>) {
    let state: tauri::State<AppState> = app.state();

    // 检查是否有结构性变化（新增或移除 provider）
    let has_structural_change = {
        let info = state.usage_info.lock().unwrap();
        let old_ids: std::collections::HashSet<&str> =
            info.iter().map(|u| u.provider_id()).collect();

        results.iter().any(|r| match &r.result {
            Ok(_) => !old_ids.contains(r.provider_id),
            Err(crate::error::AppError::Auth(_)) => old_ids.contains(r.provider_id),
            _ => false,
        })
    };

    state.merge_usage(results);

    if has_structural_change || state.menu_handles.lock().unwrap().is_none() {
        rebuild_menu(app);
    } else {
        update_menu_in_place(app);
    }
}

/// 兼容旧接口：直接用 usage_list 更新（用于登录后的刷新）
pub fn update_menu(app: &AppHandle, usage_list: Vec<Box<dyn UsageInfo>>) {
    let state: tauri::State<AppState> = app.state();

    // 直接替换 usage 数据
    {
        let mut info = state.usage_info.lock().unwrap();
        *info = usage_list;
    }

    rebuild_menu(app);
}
