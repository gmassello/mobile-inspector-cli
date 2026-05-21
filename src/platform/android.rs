use crate::error::{InspectorError, Result};
use std::process::Command;

pub struct AdbPlatform {
    serial: Option<String>,
}

impl AdbPlatform {
    pub fn new(serial: Option<String>) -> Self {
        Self { serial }
    }

    fn adb(&self) -> Command {
        let mut cmd = Command::new("adb");
        if let Some(s) = &self.serial {
            cmd.arg("-s").arg(s);
        }
        cmd
    }
}

impl super::Platform for AdbPlatform {
    fn dump_xml(&self) -> Result<String> {
        let out = self
            .adb()
            .args(["exec-out", "uiautomator", "dump", "/dev/tty"])
            .output()
            .map_err(|e| InspectorError::Adb(format!("no se pudo invocar adb: {e}")))?;

        if !out.status.success() {
            return Err(InspectorError::Adb(
                String::from_utf8_lossy(&out.stderr).into_owned(),
            ));
        }

        let raw = String::from_utf8_lossy(&out.stdout);
        extract_xml(&raw)
    }

    fn name(&self) -> &'static str {
        "android"
    }
}

fn extract_xml(raw: &str) -> Result<String> {
    let start = raw
        .find("<?xml")
        .or_else(|| raw.find("<hierarchy"))
        .ok_or_else(|| InspectorError::Adb("no se encontro XML en la salida de adb".into()))?;

    let end = raw
        .rfind("</hierarchy>")
        .map(|i| i + "</hierarchy>".len())
        .ok_or_else(|| InspectorError::Adb("falta el cierre </hierarchy>".into()))?;

    Ok(raw[start..end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_xml_with_trailing_message() {
        let raw = "<?xml version=\"1.0\"?><hierarchy rotation=\"0\"><node/></hierarchy>\r\nUI hierchary dumped to: /sdcard/window_dump.xml\r\n";
        let xml = extract_xml(raw).unwrap();
        assert!(xml.starts_with("<?xml"));
        assert!(xml.ends_with("</hierarchy>"));
    }

    #[test]
    fn fails_when_no_xml() {
        let err = extract_xml("nothing here").unwrap_err();
        assert!(matches!(err, InspectorError::Adb(_)));
    }
}
