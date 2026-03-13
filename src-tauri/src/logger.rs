use std::{env, fs, path::PathBuf};

use crate::error::{AppError, Result};

fn log_dir() -> PathBuf {
    if let Ok(home) = env::var("HOME") {
        return PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("am-i-in-debt")
            .join("logs");
    }

    env::temp_dir().join("am-i-in-debt").join("logs")
}

pub fn init_logging() -> Result<()> {
    let log_dir = log_dir();
    fs::create_dir_all(&log_dir)?;

    let log_file = log_dir.join("app.log");

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .level_for("tao", log::LevelFilter::Warn)
        .level_for("winit", log::LevelFilter::Warn)
        .chain(std::io::stdout())
        .chain(fern::log_file(log_file)?)
        .apply()
        .map_err(|e| AppError::Config(format!("初始化日志失败: {}", e)))?;

    log::info!("日志系统初始化完成");
    Ok(())
}
