use anyhow::Result;
use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct TrellisPaths {
    pub home: PathBuf,
    pub cache: PathBuf,
    pub cellar: PathBuf,
    pub receipts: PathBuf,
    pub registry: PathBuf,
    pub registry_cache: PathBuf,
    pub registry_sources: PathBuf,
    pub registry_index: PathBuf,
    pub locks: PathBuf,
    pub profiles: PathBuf,
    pub bin: PathBuf,
    pub default_registry_root: PathBuf,
}

impl TrellisPaths {
    pub fn resolve(home_override: Option<&Path>) -> Result<Self> {
        let home = match home_override {
            Some(path) => path.to_path_buf(),
            None => default_home(),
        };
        let registry = home.join("registry");
        Ok(Self {
            cache: home.join("cache"),
            cellar: home.join("cellar"),
            receipts: home.join("receipts"),
            registry_cache: registry.join("cache"),
            registry_sources: registry.join("sources.json"),
            registry_index: registry.join("index.json"),
            locks: home.join("locks"),
            profiles: home.join("profiles.json"),
            bin: home.join("bin"),
            default_registry_root: std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("packages"),
            registry,
            home,
        })
    }

    pub fn all_dirs(&self) -> Vec<&PathBuf> {
        vec![
            &self.home,
            &self.cache,
            &self.cellar,
            &self.receipts,
            &self.registry,
            &self.registry_cache,
            &self.locks,
            &self.bin,
        ]
    }
}

fn default_home() -> PathBuf {
    if let Ok(xdg) = env::var("XDG_DATA_HOME") {
        return PathBuf::from(xdg).join("trellis");
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(app_data) = env::var("APPDATA") {
            return PathBuf::from(app_data).join("Trellis");
        }
    }

    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".local/share/trellis")
}
