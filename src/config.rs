use crate::cli::{ConfigAction, ConfigArgs};
use crate::error::{InspectorError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub appium: AppiumConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppiumConfig {
    #[serde(default = "default_appium_url")]
    pub url: String,
}

impl Default for AppiumConfig {
    fn default() -> Self {
        Self {
            url: default_appium_url(),
        }
    }
}

fn default_appium_url() -> String {
    "http://localhost:4723".into()
}

pub fn config_path() -> Result<PathBuf> {
    let dir = dirs::config_dir()
        .ok_or_else(|| InspectorError::Config("no se pudo resolver el config dir".into()))?;
    Ok(dir.join("mobile-inspector").join("config.toml"))
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = fs::read_to_string(&path)?;
        toml::from_str(&raw).map_err(|e| InspectorError::Config(e.to_string()))
    }

    pub fn save(&self) -> Result<()> {
        let path = config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let raw =
            toml::to_string_pretty(self).map_err(|e| InspectorError::Config(e.to_string()))?;
        fs::write(&path, raw)?;
        Ok(())
    }
}

pub fn handle_config(args: ConfigArgs) -> anyhow::Result<()> {
    let mut cfg = Config::load()?;
    match args.action {
        ConfigAction::Path => {
            println!("{}", config_path()?.display());
        }
        ConfigAction::Get { key } => {
            let v = match key.as_str() {
                "appium.url" => cfg.appium.url.clone(),
                _ => return Err(anyhow::anyhow!("clave desconocida: {key}")),
            };
            println!("{v}");
        }
        ConfigAction::Set { key, value } => {
            match key.as_str() {
                "appium.url" => cfg.appium.url = value,
                _ => return Err(anyhow::anyhow!("clave desconocida: {key}")),
            }
            cfg.save()?;
            println!("ok");
        }
    }
    Ok(())
}
