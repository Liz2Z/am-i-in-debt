use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem, CheckMenuItem},
    AppHandle, Manager, Wry,
};

use crate::provider::UsageInfo;
use crate::providers::PROVIDERS;
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

fn build_empty_menu(app: &AppHandle, selected_provider_id: Option<String>) -> Menu<Wry> {
    let header = MenuItem::with_id(app, "header", "Am I In Debt ?", true, None::<&str>).unwrap();
    let sep1 = PredefinedMenuItem::separator(app).unwrap();
    
    let mut items: Vec<Box<dyn tauri::menu::IsMenuItem<Wry>>> = vec![
        Box::new(header),
        Box::new(sep1),
    ];
    
    for provider in PROVIDERS.iter() {
        let is_checked = selected_provider_id.as_deref() == Some(provider.id());
        let item = CheckMenuItem::with_id(
            app,
            format!("select-{}", provider.id()),
            format!("{} Coding Plan", provider.display_name()),
            true,
            is_checked,
            None::<&str>,
        ).unwrap();
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
    selected_provider_id: Option<String>,
) -> Menu<Wry> {
    let header = MenuItem::with_id(app, "header", "Am I In Debt ?", true, None::<&str>).unwrap();
    let sep1 = PredefinedMenuItem::separator(app).unwrap();

    let mut items: Vec<Box<dyn tauri::menu::IsMenuItem<Wry> + '_>> = vec![
        Box::new(header),
        Box::new(sep1),
    ];

    for provider in PROVIDERS.iter() {
        if let Some(usage) = usage_list.iter().find(|u| u.provider_id() == provider.id()) {
            let is_selected = selected_provider_id.as_deref() == Some(provider.id());
            let menu_items = usage.render_menu_items(app, is_selected);
            for item in menu_items {
                items.push(item);
            }
        } else {
            items.push(Box::new(MenuItem::with_id(
                app,
                format!("login-{}", provider.id()),
                format!("登录{} Coding Plan", provider.display_name()),
                true,
                None::<&str>,
            ).unwrap()));
        }
    }

    items.push(Box::new(PredefinedMenuItem::separator(app).unwrap()));
    items.push(Box::new(MenuItem::with_id(app, "refresh", format!("刷新{}", update_time_suffix), true, None::<&str>).unwrap()));

    for provider in PROVIDERS.iter() {
        if usage_list.iter().any(|u| u.provider_id() == provider.id()) {
            items.push(Box::new(MenuItem::with_id(
                app,
                format!("relogin-{}", provider.id()),
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
