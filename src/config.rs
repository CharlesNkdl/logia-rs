use crate::ui::theme::ColorScheme;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
    pub key_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AppConfig {
    pub servers: Vec<ServerConfig>,
    pub local_paths: Option<Vec<String>>,
    pub color_scheme: Option<ColorScheme>,
}

impl AppConfig {
    fn config_path() -> Option<PathBuf> {
        let proj_dirs = directories::ProjectDirs::from("com", "logia", "logia-rs")?;
        Some(proj_dirs.config_dir().join("config.toml"))
    }
    pub fn load() -> Self {
        let path = match Self::config_path() {
            Some(p) => p,
            None => return Self::default(),
        };
        if let Ok(contents) = fs::read_to_string(&path) {
            match toml::from_str(&contents) {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("Config parse error: {}", e);
                    Self::default()
                }
            }
        } else {
            Self::default()
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path().ok_or("Could not find configuration directory")?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let toml_string = toml::to_string_pretty(self)?;
        fs::write(path, toml_string)?;
        Ok(())
    }
}
