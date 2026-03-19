use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::error::{AppError, Result};
use crate::provider::Provider;

/// 获取开发环境中的 sidecar 路径
fn get_dev_sidecar_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("bin")
        .join("get-cookies")
}

pub async fn run_login_script(app: &tauri::AppHandle, provider: &dyn Provider) -> Result<()> {
    use tauri_plugin_shell::ShellExt;

    let output_dir = provider.data_dir();

    log::info!(
        "登录流程开始: provider={}, output_dir={}",
        provider.id(),
        output_dir.display()
    );
    fs::create_dir_all(&output_dir)?;

    let platform_arg = provider.login_script_arg();
    let is_dev = tauri::is_dev();

    let (success, stdout, stderr) = if is_dev {
        // 开发环境：直接使用 std::process::Command
        let sidecar_path = get_dev_sidecar_path();
        log::info!("开发环境: 使用 sidecar: {}, 参数: {}", sidecar_path.display(), platform_arg);

        let output = Command::new(&sidecar_path)
            .arg(platform_arg)
            .env("COOKIES_OUTPUT_PATH", output_dir.to_string_lossy().to_string())
            .output()
            .map_err(|e| AppError::Config(format!("执行 sidecar 失败: {}", e)))?;

        (
            output.status.success(),
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        )
    } else {
        // 生产环境：使用 Tauri shell 插件
        log::info!("生产环境: 使用 shell 插件 sidecar, 参数: {}", platform_arg);

        let output = app
            .shell()
            .sidecar("bin/get-cookies")
            .map_err(|e| AppError::Config(format!("无法找到 sidecar: {}", e)))?
            .args([platform_arg])
            .env("COOKIES_OUTPUT_PATH", output_dir.to_string_lossy().to_string())
            .output()
            .await
            .map_err(|e| AppError::Config(format!("执行 sidecar 失败: {}", e)))?;

        (
            output.status.success(),
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        )
    };

    if !success {
        log::error!("登录 sidecar 执行失败: status={}", stdout);
        return Err(AppError::Config(format!(
            "登录失败: {}\n{}",
            stdout, stderr
        )));
    }

    if !stdout.trim().is_empty() {
        log::info!("sidecar stdout:\n{}", stdout);
    }

    if !stderr.trim().is_empty() {
        log::warn!("sidecar stderr:\n{}", stderr);
    }

    log::info!("登录流程结束: provider={}", provider.id());
    Ok(())
}
