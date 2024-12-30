//! Loads units from various sources

use async_std::path::PathBuf;
use anyhow::{Result, Context};

/// Loads units from the local filesystem
#[derive(Clone)]
pub struct Loader {
    search_paths: Vec<PathBuf>,
}

impl Loader {
    pub fn new(search_paths: Vec<PathBuf>) -> Self {
        Self { search_paths }
    }

    /// Gets the script for a given unit from the local filesystem, searching
    /// the configured unit directories in order
    pub async fn get_script(&self, unit_name: &str) -> Result<String> {
        for dir in &self.search_paths {
            let script_path = dir.join(unit_name);

            if script_path.exists().await {
                return std::fs::read_to_string(&script_path).
                    context(format!("Could not read script: {}", script_path.to_string_lossy()));
            }
        }

        Err(anyhow::anyhow!("Script not found: {}, search_paths: {:?}", unit_name, self.search_paths))
    }
}
