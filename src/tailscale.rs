use std::process::Command;
use std::collections::HashSet;
use color_eyre::eyre::{eyre, Result};

use crate::config_loader::{self, AppConfig};

pub fn get_servers() -> Result<Vec<super::Server>> { // Note: `super::Server` here
  let config: &AppConfig = config_loader::use_config()
    .map_err(|e| eyre!("Failed to load application config: {}", e))?;

  let allowed_names_set: HashSet<String> = config.allowed.values()
    .flatten()
    .cloned()
    .collect();

  let output = Command::new("tailscale")
    .arg("status")
    .output()
    .map_err(|e| eyre!("Failed to execute 'tailscale status' command: {}", e))?;

  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    return Err(eyre!("'tailscale status' command failed: {}", stderr));
  }

  let status_text = String::from_utf8_lossy(&output.stdout);
  let mut discovered_servers: Vec<super::Server> = Vec::new(); // Note: `super::Server` here

  for line in status_text.lines() {
    if line.trim().is_empty() || line.trim().starts_with('#') {
      continue;
    }

    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.len() >= 2 {
      let ip = parts[0].to_string();
      let name_raw = parts[1];

      let name = name_raw.split('.').next().unwrap_or(name_raw).to_string();

      if allowed_names_set.contains(&name) {
        discovered_servers.push(super::Server { // Note: `super::Server` here
          name,
          ip,
        });
      }
    }
  }

  Ok(discovered_servers)
}