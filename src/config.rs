use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RewriteTarget {
    Fx,
    Vx,
}

impl RewriteTarget {
    pub fn host(self) -> &'static str {
        match self {
            Self::Fx => "fxtwitter.com",
            Self::Vx => "vxtwitter.com",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfig {
    pub enabled: bool,
    pub target: RewriteTarget,
    pub launch_on_startup: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            target: RewriteTarget::Fx,
            launch_on_startup: true,
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        let path = config_path();
        Self::load_from_path(&path).unwrap_or_default()
    }

    pub fn load_from_path(path: &Path) -> io::Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let text = fs::read_to_string(path)?;
        serde_json::from_str(&text).map_err(io::Error::other)
    }

    pub fn save(&self) -> io::Result<()> {
        let path = config_path();
        self.save_to_path(&path)
    }

    pub fn save_to_path(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let text = serde_json::to_string_pretty(self).map_err(io::Error::other)?;
        fs::write(path, text)
    }
}

pub fn config_path() -> PathBuf {
    if let Some(appdata) = std::env::var_os("APPDATA") {
        PathBuf::from(appdata).join("fix-x").join("config.json")
    } else {
        PathBuf::from("config.json")
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::{AppConfig, RewriteTarget};

    #[test]
    fn missing_config_returns_default() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.json");

        let config = AppConfig::load_from_path(&path).unwrap();

        assert_eq!(config, AppConfig::default());
    }

    #[test]
    fn save_and_reload_round_trips() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.json");
        let config = AppConfig {
            enabled: false,
            target: RewriteTarget::Vx,
            launch_on_startup: false,
        };

        config.save_to_path(&path).unwrap();
        let loaded = AppConfig::load_from_path(&path).unwrap();

        assert_eq!(loaded, config);
    }
}
