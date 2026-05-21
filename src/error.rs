use thiserror::Error;

#[derive(Debug, Error)]
pub enum InspectorError {
    #[error("adb error: {0}")]
    Adb(String),

    #[error("appium error: {0}")]
    Appium(String),

    #[error("xml parse error: {0}")]
    XmlParse(String),

    #[error("xpath error: {0}")]
    XPath(String),

    #[error("filter error: {0}")]
    Filter(String),

    #[error("config error: {0}")]
    Config(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, InspectorError>;
