use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem, CheckMenuItem},
    AppHandle, Manager, Wry,
};

use crate::models::{format_progress_bar, Provider, UsageInfo, ZhipuUsageInfo, KimiUsageInfo};
use crate::state::AppState;
use crate::provider_switch::get_current_selected_provider;

pub fn build_menu(app: &AppHandle, usage_list: &[UsageInfo], update_time_suffix: &str) -> Menu<Wry> {
    let selected_provider = get_current_selected_provider();
    
    if usage_list.is_empty() {
        build_empty_menu(app, selected_provider)
    } else {
        build_usage_menu(app, usage_list, update_time_suffix, selected_provider)
    }
}

fn build_empty_menu(app: &AppHandle, selected_provider: Option<Provider>) -> Menu<Wry> {
    let header = MenuItem::with_id(app, "header", "Am I In Debt ?", true, None::<&str>).unwrap();
    let sep1 = PredefinedMenuItem::separator(app).unwrap();
    
    let mut items: Vec<Box<dyn tauri::menu::IsMenuItem<Wry>>> = vec![
        Box::new(header),
        Box::new(sep1),
    ];
    
    for provider in Provider::ALL {
        let is_checked = selected_provider == Some(provider);
        let item = CheckMenuItem::with_id(
            app,
            format!("select-{}", provider.provider_id()),
            format!("{} Coding Plan", provider.display_name()),
            true,
            is_checked,
            None::<&str>,
        ).unwrap();
        items.push(Box::new(item));
    }
    
    let sep2 = PredefinedMenuItem::separator(app).unwrap();
    items.push(Box::new(sep2));
    
    for provider in Provider::ALL {
        let item = MenuItem::with_id(
            app,
            format!("login-{}", provider.provider_id()),
            format!("登录{} Coding Plan", provider.display_name()),
            true,
            None::<&str>,
        ).unwrap();
        items.push(Box::new(item));
    }
    
    let sep3 = PredefinedMenuItem::separator(app).unwrap();
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>).unwrap();
    items.push(Box::new(sep3));
    items.push(Box::new(quit));

    let items_refs: Vec<&dyn tauri::menu::IsMenuItem<Wry>> = items.iter().map(|item| item.as_ref()).collect();
    Menu::with_items(app, &items_refs).unwrap()
}

fn build_usage_menu(
    app: &AppHandle,
    usage_list: &[UsageInfo],
    update_time_suffix: &str,
    selected_provider: Option<Provider>,
) -> Menu<Wry> {
    let header = MenuItem::with_id(app, "header", "Am I In Debt ?", true, None::<&str>).unwrap();
    let sep1 = PredefinedMenuItem::separator(app).unwrap();

    let mut items: Vec<Box<dyn tauri::menu::IsMenuItem<Wry> + '_>> = vec![
        Box::new(header),
        Box::new(sep1),
    ];

    for provider in Provider::ALL {
        if let Some(usage) = usage_list.iter().find(|u| u.provider_id() == provider.provider_id()) {
            match usage {
                UsageInfo::Zhipu(info) => {
                    add_zhipu_menu_items(app, &mut items, provider, info, selected_provider);
                }
                UsageInfo::Kimi(info) => {
                    add_kimi_menu_items(app, &mut items, provider, info, selected_provider);
                }
            }
        } else {
            items.push(Box::new(MenuItem::with_id(
                app,
                format!("login-{}", provider.provider_id()),
                format!("登录{} Coding Plan", provider.display_name()),
                true,
                None::<&str>,
            ).unwrap()));
        }
    }

    items.push(Box::new(PredefinedMenuItem::separator(app).unwrap()));
    items.push(Box::new(MenuItem::with_id(app, "refresh", format!("刷新{}", update_time_suffix), true, None::<&str>).unwrap()));

    for provider in Provider::ALL {
        if usage_list.iter().any(|u| u.provider() == provider) {
            items.push(Box::new(MenuItem::with_id(
                app,
                format!("relogin-{}", provider.provider_id()),
                format!("重新登录{}", provider.display_name()),
                true,
                None::<&str>,
            ).unwrap()));
        }
    }

    items.push(Box::new(PredefinedMenuItem::separator(app).unwrap()));
    items.push(Box::new(MenuItem::with_id(app, "quit", "退出", true, None::<&str>).unwrap()));

    let items_refs: Vec<&dyn tauri::menu::IsMenuItem<Wry>> = items.iter().map(|item| item.as_ref()).collect();
    Menu::with_items(app, &items_refs).unwrap()
}

fn add_zhipu_menu_items<'a>(
    app: &'a AppHandle,
    items: &mut Vec<Box<dyn tauri::menu::IsMenuItem<Wry> + 'a>>,
    provider: Provider,
    info: &ZhipuUsageInfo,
    selected_provider: Option<Provider>,
) {
    let is_selected = selected_provider == Some(provider);
    
    items.push(Box::new(CheckMenuItem::with_id(
        app,
        format!("select-{}", provider.provider_id()),
        format!("{} Coding Plan", provider.display_name()),
        true,
        is_selected,
        None::<&str>,
    ).unwrap()));

    items.push(Box::new(MenuItem::with_id(
        app,
        format!("{}-token-title", provider.provider_id()),
        format!("Token 额度（每 {} 小时）", info.token_hours),
        false,
        None::<&str>,
    ).unwrap()));
    items.push(Box::new(MenuItem::with_id(
        app,
        format!("{}-token-bar", provider.provider_id()),
        format_progress_bar(info.token_percentage),
        false,
        None::<&str>,
    ).unwrap()));
    items.push(Box::new(MenuItem::with_id(
        app,
        format!("{}-token-reset", provider.provider_id()),
        format!("重置: {}", info.token_reset_time),
        false,
        None::<&str>,
    ).unwrap()));

    items.push(Box::new(MenuItem::with_id(
        app,
        format!("{}-sep", provider.provider_id()),
        "-".repeat(25),
        false,
        None::<&str>,
    ).unwrap()));

    items.push(Box::new(MenuItem::with_id(
        app,
        format!("{}-mcp-title", provider.provider_id()),
        "MCP 额度（每月）",
        false,
        None::<&str>,
    ).unwrap()));
    items.push(Box::new(MenuItem::with_id(
        app,
        format!("{}-mcp-bar", provider.provider_id()),
        format_progress_bar(info.mcp_percentage as f64),
        false,
        None::<&str>,
    ).unwrap()));
    items.push(Box::new(MenuItem::with_id(
        app,
        format!("{}-mcp-detail", provider.provider_id()),
        format!("搜索: {} | 网页: {} | 阅读: {}", info.mcp_search, info.mcp_web, info.mcp_zread),
        false,
        None::<&str>,
    ).unwrap()));
    items.push(Box::new(MenuItem::with_id(
        app,
        format!("{}-mcp-reset", provider.provider_id()),
        format!("重置: {}", info.mcp_reset_time),
        false,
        None::<&str>,
    ).unwrap()));
    items.push(Box::new(PredefinedMenuItem::separator(app).unwrap()));
}

fn add_kimi_menu_items<'a>(
    app: &'a AppHandle,
    items: &mut Vec<Box<dyn tauri::menu::IsMenuItem<Wry> + 'a>>,
    provider: Provider,
    info: &KimiUsageInfo,
    selected_provider: Option<Provider>,
) {
    let is_selected = selected_provider == Some(provider);
    
    items.push(Box::new(CheckMenuItem::with_id(
        app,
        format!("select-{}", provider.provider_id()),
        format!("{} Coding Plan", provider.display_name()),
        true,
        is_selected,
        None::<&str>,
    ).unwrap()));

    items.push(Box::new(MenuItem::with_id(
        app,
        format!("{}-hourly-title", provider.provider_id()),
        format!("Token 额度（每 {} 小时）", info.hourly_window),
        false,
        None::<&str>,
    ).unwrap()));
    items.push(Box::new(MenuItem::with_id(
        app,
        format!("{}-hourly-bar", provider.provider_id()),
        format_progress_bar(info.hourly_percentage),
        false,
        None::<&str>,
    ).unwrap()));
    items.push(Box::new(MenuItem::with_id(
        app,
        format!("{}-hourly-reset", provider.provider_id()),
        format!("重置: {}", info.hourly_reset_time),
        false,
        None::<&str>,
    ).unwrap()));

    items.push(Box::new(MenuItem::with_id(
        app,
        format!("{}-sep", provider.provider_id()),
        "-".repeat(25),
        false,
        None::<&str>,
    ).unwrap()));

    items.push(Box::new(MenuItem::with_id(
        app,
        format!("{}-weekly-title", provider.provider_id()),
        "Token 额度（每周）",
        false,
        None::<&str>,
    ).unwrap()));
    items.push(Box::new(MenuItem::with_id(
        app,
        format!("{}-weekly-bar", provider.provider_id()),
        format_progress_bar(info.weekly_percentage),
        false,
        None::<&str>,
    ).unwrap()));
    items.push(Box::new(MenuItem::with_id(
        app,
        format!("{}-weekly-reset", provider.provider_id()),
        format!("重置: {}", info.weekly_reset_time),
        false,
        None::<&str>,
    ).unwrap()));
    items.push(Box::new(PredefinedMenuItem::separator(app).unwrap()));
}

pub fn update_menu(app: &AppHandle, usage_list: &[UsageInfo]) {
    let state: tauri::State<AppState> = app.state();
    state.update_usage(usage_list.to_vec());
    state.update_time();

    let update_time_suffix = state.get_update_time_suffix();

    let menu = build_menu(app, usage_list, &update_time_suffix);

    if let Some(tray) = state.get_tray() {
        let _ = tray.set_menu(Some(menu));
    }
}
