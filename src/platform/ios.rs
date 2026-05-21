use crate::error::{InspectorError, Result};
use reqwest::blocking::Client;
use serde_json::Value;
use std::time::Duration;

pub struct AppiumPlatform {
    base_url: String,
    session: Option<String>,
    client: Client,
}

impl AppiumPlatform {
    pub fn new(base_url: String, session: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("build http client");
        Self {
            base_url,
            session,
            client,
        }
    }

    fn base(&self) -> &str {
        self.base_url.trim_end_matches('/')
    }

    fn get_json(&self, path: &str) -> Result<Value> {
        let url = format!("{}{}", self.base(), path);
        let resp = self
            .client
            .get(&url)
            .send()
            .map_err(|e| InspectorError::Appium(format!("GET {path} fallo: {e}")))?;
        if !resp.status().is_success() {
            return Err(InspectorError::Appium(format!(
                "appium respondio {} en {path}: {}",
                resp.status(),
                resp.text().unwrap_or_default()
            )));
        }
        Ok(resp.json()?)
    }

    fn resolve_session(&self) -> Result<String> {
        if let Some(s) = &self.session {
            return Ok(s.clone());
        }
        let body = self.get_json("/sessions")?;
        let arr = body
            .get("value")
            .and_then(|v| v.as_array())
            .ok_or_else(|| InspectorError::Appium("respuesta /sessions malformada".into()))?;

        match arr.len() {
            0 => Err(InspectorError::Appium(
                "no hay sesiones appium activas".into(),
            )),
            1 => arr[0]
                .get("id")
                .and_then(|v| v.as_str())
                .map(String::from)
                .ok_or_else(|| InspectorError::Appium("sesion sin id".into())),
            n => Err(InspectorError::Appium(format!(
                "{n} sesiones activas; pasa --session <id>"
            ))),
        }
    }
}

impl super::Platform for AppiumPlatform {
    fn dump_xml(&self) -> Result<String> {
        let sid = self.resolve_session()?;
        let body = self.get_json(&format!("/session/{sid}/source"))?;
        body.get("value")
            .and_then(|v| v.as_str())
            .map(String::from)
            .ok_or_else(|| InspectorError::Appium("falta 'value' en la respuesta appium".into()))
    }

    fn name(&self) -> &'static str {
        "ios"
    }
}
