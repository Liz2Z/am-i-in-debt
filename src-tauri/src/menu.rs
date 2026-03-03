use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem, CheckMenuItem},
    AppHandle, Manager, Wry,
};

use crate::models::{format_progress_bar, CodingPlan, UsageInfo, ZhipuUsageInfo, KimiUsageInfo};
use crate::state::AppState;
use crate::provider_switch::get_current_selected_plan;

pub fn build_menu(app: &AppHandle, usage_list: &[UsageInfo], update_time_suffix: &str) -> Menu<Wry> {
    let selected_plan = get_current_selected_plan();
    
    if usage_list.is_empty() {
        build_empty_menu(app, selected_plan)
    } else {
        build_usage_menu(app, usage_list, update_time_suffix, selected_plan)
    }
}

fn build_empty_menu(app: &AppHandle, selected_plan: Option<CodingPlan>) -> Menu<Wry> {
    let header = MenuItem::with_id(app, "header", "Am I In Debt ?", true, None::<&str>).unwrap();
    let sep1 = PredefinedMenuItem::separator(app).unwrap();
    
    let zhipu_checked = selected_plan == Some(CodingPlan::Zhipu);
    let kimi_checked = selected_plan == Some(CodingPlan::Kimi);
    
    let select_zhipu = CheckMenuItem::with_id(app, "select-zhipu", "智谱 Coding Plan", true, zhipu_checked, None::<&str>).unwrap();
    let select_kimi = CheckMenuItem::with_id(app, "select-kimi", "Kimi Coding Plan", true, kimi_checked, None::<&str>).unwrap();
    
    let sep2 = PredefinedMenuItem::separator(app).unwrap();
    let login_zhipu = MenuItem::with_id(app, "login-zhipu", "登录智谱 Coding Plan", true, None::<&str>).unwrap();
    let login_kimi = MenuItem::with_id(app, "login-kimi", "登录 Kimi Coding Plan", true, None::<&str>).unwrap();
    let sep3 = PredefinedMenuItem::separator(app).unwrap();
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>).unwrap();

    Menu::with_items(
        app,
        &[&header as &dyn tauri::menu::IsMenuItem<Wry>, &sep1, &select_zhipu, &select_kimi, &sep2, &login_zhipu, &login_kimi, &sep3, &quit],
    ).unwrap()
}

fn build_usage_menu(
    app: &AppHandle,
    usage_list: &[UsageInfo],
    update_time_suffix: &str,
    selected_plan: Option<CodingPlan>,
) -> Menu<Wry> {
    let header = MenuItem::with_id(app, "header", "Am I In Debt ?", true, None::<&str>).unwrap();
    let sep1 = PredefinedMenuItem::separator(app).unwrap();

    let mut items: Vec<Box<dyn tauri::menu::IsMenuItem<Wry> + '_>> = vec![
        Box::new(header),
        Box::new(sep1),
    ];

    for plan in [CodingPlan::Zhipu, CodingPlan::Kimi] {
        if let Some(usage) = usage_list.iter().find(|u| u.plan_id() == plan.id()) {
            match usage {
                UsageInfo::Zhipu(info) => {
                    add_zhipu_menu_items(app, &mut items, plan, info, selected_plan);
                }
                UsageInfo::Kimi(info) => {
                    add_kimi_menu_items(app, &mut items, plan, info, selected_plan);
                }
            }
        } else {
            items.push(Box::new(MenuItem::with_id(
                app,
                format!("login-{}", plan.id()),
                format!("登录{} Coding Plan", plan.name()),
                true,
                None::<&str>,
            ).unwrap()));
        }
    }

    items.push(Box::new(PredefinedMenuItem::separator(app).unwrap()));
    items.push(Box::new(MenuItem::with_id(app, "refresh", format!("刷新{}", update_time_suffix), true, None::<&str>).unwrap()));

    let zhipu_logged_in = usage_list.iter().any(|u| u.is_zhipu());
    let kimi_logged_in = usage_list.iter().any(|u| u.is_kimi());
    
    if zhipu_logged_in {
        items.push(Box::new(MenuItem::with_id(app, "relogin-zhipu", "重新登录智谱", true, None::<&str>).unwrap()));
    }
    if kimi_logged_in {
        items.push(Box::new(MenuItem::with_id(app, "relogin-kimi", "重新登录 Kimi", true, None::<&str>).unwrap()));
    }

    items.push(Box::new(PredefinedMenuItem::separator(app).unwrap()));
    items.push(Box::new(MenuItem::with_id(app, "quit", "退出", true, None::<&str>).unwrap()));

    let items_refs: Vec<&dyn tauri::menu::IsMenuItem<Wry>> = items.iter().map(|item| item.as_ref()).collect();
    Menu::with_items(app, &items_refs).unwrap()
}

fn add_zhipu_menu_items<'a>(
    app: &'a AppHandle,
    items: &mut Vec<Box<dyn tauri::menu::IsMenuItem<Wry> + 'a>>,
    plan: CodingPlan,
    info: &ZhipuUsageInfo,
    selected_plan: Option<CodingPlan>,
) {
    let is_selected = selected_plan == Some(plan);
    
    items.push(Box::new(CheckMenuItem::with_id(
        app,
        format!("select-{}", plan.id()),
        format!("{} Coding Plan", plan.name()),
        true,
        is_selected,
        None::<&str>,
    ).unwrap()));

    items.push(Box::new(MenuItem::with_id(
        app,
        format!("{}-token-title", plan.id()),
        format!("Token 额度（每 {} 小时）", info.token_hours),
        false,
        None::<&str>,
    ).unwrap()));
    items.push(Box::new(MenuItem::with_id(
        app,
        format!("{}-token-bar", plan.id()),
        format_progress_bar(info.token_percentage),
        false,
        None::<&str>,
    ).unwrap()));
    items.push(Box::new(MenuItem::with_id(
        app,
        format!("{}-token-reset", plan.id()),
        format!("重置: {}", info.token_reset_time),
        false,
        None::<&str>,
    ).unwrap()));

    items.push(Box::new(MenuItem::with_id(
        app,
        format!("{}-sep", plan.id()),
        "-".repeat(25),
        false,
        None::<&str>,
    ).unwrap()));

    items.push(Box::new(MenuItem::with_id(
        app,
        format!("{}-mcp-title", plan.id()),
        "MCP 额度（每月）",
        false,
        None::<&str>,
    ).unwrap()));
    items.push(Box::new(MenuItem::with_id(
        app,
        format!("{}-mcp-bar", plan.id()),
        format_progress_bar(info.mcp_percentage as f64),
        false,
        None::<&str>,
    ).unwrap()));
    items.push(Box::new(MenuItem::with_id(
        app,
        format!("{}-mcp-detail", plan.id()),
        format!("搜索: {} | 网页: {} | 阅读: {}", info.mcp_search, info.mcp_web, info.mcp_zread),
        false,
        None::<&str>,
    ).unwrap()));
    items.push(Box::new(MenuItem::with_id(
        app,
        format!("{}-mcp-reset", plan.id()),
        format!("重置: {}", info.mcp_reset_time),
        false,
        None::<&str>,
    ).unwrap()));
    items.push(Box::new(PredefinedMenuItem::separator(app).unwrap()));
}

fn add_kimi_menu_items<'a>(
    app: &'a AppHandle,
    items: &mut Vec<Box<dyn tauri::menu::IsMenuItem<Wry> + 'a>>,
    plan: CodingPlan,
    info: &KimiUsageInfo,
    selected_plan: Option<CodingPlan>,
) {
    let is_selected = selected_plan == Some(plan);
    
    items.push(Box::new(CheckMenuItem::with_id(
        app,
        format!("select-{}", plan.id()),
        format!("{} Coding Plan", plan.name()),
        true,
        is_selected,
        None::<&str>,
    ).unwrap()));

    items.push(Box::new(MenuItem::with_id(
        app,
        format!("{}-hourly-title", plan.id()),
        format!("Token 额度（每 {} 小时）", info.hourly_window),
        false,
        None::<&str>,
    ).unwrap()));
    items.push(Box::new(MenuItem::with_id(
        app,
        format!("{}-hourly-bar", plan.id()),
        format_progress_bar(info.hourly_percentage),
        false,
        None::<&str>,
    ).unwrap()));
    items.push(Box::new(MenuItem::with_id(
        app,
        format!("{}-hourly-reset", plan.id()),
        format!("重置: {}", info.hourly_reset_time),
        false,
        None::<&str>,
    ).unwrap()));

    items.push(Box::new(MenuItem::with_id(
        app,
        format!("{}-sep", plan.id()),
        "-".repeat(25),
        false,
        None::<&str>,
    ).unwrap()));

    items.push(Box::new(MenuItem::with_id(
        app,
        format!("{}-weekly-title", plan.id()),
        "Token 额度（每周）",
        false,
        None::<&str>,
    ).unwrap()));
    items.push(Box::new(MenuItem::with_id(
        app,
        format!("{}-weekly-bar", plan.id()),
        format_progress_bar(info.weekly_percentage),
        false,
        None::<&str>,
    ).unwrap()));
    items.push(Box::new(MenuItem::with_id(
        app,
        format!("{}-weekly-reset", plan.id()),
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
