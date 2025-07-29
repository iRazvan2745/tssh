use color_eyre::eyre::{eyre, Result}; // Import eyre and Result from color_eyre
use serde::{Deserialize, Serialize}; // Add Serialize for writing default config
use std::collections::HashMap;
use std::fs::{self, File}; // Import File
use std::io::Read; // Import Read trait for file.read_to_string
use once_cell::sync::OnceCell; // Import OnceCell

#[derive(Debug, Deserialize, Serialize)] // Add Serialize to write default config
pub struct AppConfig { // Renamed from Config to AppConfig for consistency
  pub user: String,
  pub allowed: HashMap<String, Vec<String>>,
}

// Default implementation for AppConfig when the file doesn't exist
impl Default for AppConfig {
  fn default() -> Self {
    AppConfig {
      user: "default_user".to_string(), // Provide a meaningful default user
      allowed: HashMap::new(), // Initialize as an empty HashMap
    }
  }
}

static APP_CONFIG_INSTANCE: OnceCell<AppConfig> = OnceCell::new(); // Renamed to avoid conflict with function name

pub fn use_config() -> Result<&'static AppConfig> { // Correct return type for color_eyre::Result
  APP_CONFIG_INSTANCE.get_or_try_init(|| {
    let config_dir = dirs::config_dir()
      .ok_or_else(|| eyre!("Could not find config directory"))? // Use eyre! macro for errors
      .join("tssh");
    let config_file_path = config_dir.join("config.json");

    if !config_file_path.exists() {
      // Create the directory if it doesn't exist
      fs::create_dir_all(&config_dir)
        .map_err(|e| {
          eyre!(
                        "Error creating config directory {}: {}",
                        config_dir.display(),
                        e
                    )
        })?;

      // Create the file with default empty values
      let default_config = AppConfig::default();
      let default_json = serde_json::to_string_pretty(&default_config)
        .map_err(|e| eyre!("Error serializing default config: {}", e))?;

      fs::write(&config_file_path, default_json)
        .map_err(|e| {
          eyre!(
                        "Error writing default config to file {}: {}",
                        config_file_path.display(),
                        e
                    )
        })?;

      // Directly return the default_config value,
      // OnceCell will take ownership.
      return Ok(default_config);
    }

    let mut file = File::open(&config_file_path)
      .map_err(|e| {
        eyre!(
                    "Error opening config file {}: {}",
                    config_file_path.display(),
                    e
                )
      })?;

    let mut json_content = String::new();
    file.read_to_string(&mut json_content)
      .map_err(|e| {
        eyre!(
                    "Error reading config file {}: {}",
                    config_file_path.display(),
                    e
                )
      })?;

    let config: AppConfig = serde_json::from_str(&json_content)
      .map_err(|e| {
        eyre!(
                    "Error parsing JSON from config file {}: {}",
                    config_file_path.display(),
                    e
                )
      })?;
    Ok(config)
  })
}