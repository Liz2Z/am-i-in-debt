use std::fs;
use std::path::PathBuf;

use crate::error::{AppError, Result};
use crate::provider::Provider;

fn get_sidecar_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("bin")
        .join("get-cookies")
}

fn get_sidecar_path_with_target() -> Option<PathBuf> {
    let base = get_sidecar_path();
    let target_triple = tauri::utils::platform::target_triple().ok()?;
    Some(base.with_file_name(format!("get-cookies-{}", target_triple)))
}

pub fn run_login_script(provider: &dyn Provider) -> Result<()> {
    let output_dir = provider.data_dir();

    fs::create_dir_all(&output_dir)?;

    let sidecar_path = get_sidecar_path();
    let sidecar_path = if sidecar_path.exists() {
        sidecar_path
    } else {
        let path_with_target = get_sidecar_path_with_target()
            .filter(|p| p.exists())
            .ok_or_else(|| AppError::Config(format!(
                "Sidecar 文件不存在: {:?}",
                get_sidecar_path()
            )))?;
        path_with_target
    };

    let platform_arg = provider.login_script_arg();

    let output = std::process::Command::new(&sidecar_path)
        .arg(platform_arg)
        .env("COOKIES_OUTPUT_PATH", output_dir.to_string_lossy().to_string())
        .output()
        .map_err(|e| AppError::Config(format!("执行 sidecar 失败: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(AppError::Config(format!("登录失败: {}\n{}", stdout, stderr)));
    }

    log::info!("{}", String::from_utf8_lossy(&output.stdout));
    log::error!("{}", String::from_utf8_lossy(&output.stderr));

    Ok(())
}
