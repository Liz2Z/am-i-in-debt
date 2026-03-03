use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Http(String),
    Auth(String),
    Parse(String),
    Config(String),
    Unknown(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(e) => write!(f, "IO 错误: {}", e),
            AppError::Json(e) => write!(f, "JSON 解析错误: {}", e),
            AppError::Http(msg) => write!(f, "HTTP 请求失败: {}", msg),
            AppError::Auth(msg) => write!(f, "认证失败: {}", msg),
            AppError::Parse(msg) => write!(f, "解析错误: {}", msg),
            AppError::Config(msg) => write!(f, "配置错误: {}", msg),
            AppError::Unknown(msg) => write!(f, "未知错误: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Json(e)
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
