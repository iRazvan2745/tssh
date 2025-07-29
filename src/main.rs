use color_eyre::eyre::{eyre, Result};
use inquire::Select;
use std::process::Command;
use std::env;

mod config_loader;
mod tailscale;

#[derive(Debug, Clone)]
pub struct Server {
    pub name: String,
    pub ip: String,
}

impl std::fmt::Display for Server {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.ip)
    }
}

use config_loader::AppConfig;

enum ServerSelectionAction<'a> {
    SshInto(&'a Server),
    GoBack,
    ExitApp,
}

fn ssh_into_server(server: &Server, config_user: &str) -> Result<()> {
    println!("\nAttempting to SSH into: {} ({})", server.name, server.ip);
    let arg = format!("{}@{}", config_user, server.ip);

    let status = Command::new("ssh")
        .arg("-o StrictHostKeyChecking=no")
        .arg("-o UserKnownHostsFile=/dev/null")
        .arg(arg)
        .status()?;

    if !status.success() {
        return Err(eyre!("SSH command failed with status: {}", status));
    }
    Ok(())
}

fn select_category(
    config: &AppConfig,
    cli_category_input: Option<&str>,
) -> Result<Option<String>> {
    let mut categories: Vec<String> = config.allowed.keys().cloned().collect();
    categories.sort();
    categories.push("Exit Application".to_string());

    if let Some(input_category) = cli_category_input {
        if categories.contains(&input_category.to_string()) {
            return Ok(Some(input_category.to_string()));
        } else {
            println!(
                "Category '{}' not found. Please select from the list.",
                input_category
            );
        }
    }

    let ans = Select::new("Select a server category:", categories).prompt();

    match ans {
        Ok(category) if category == "Exit Application" => Ok(None),
        Ok(category) => Ok(Some(category)),
        Err(_) => Ok(None), // Treat any error (e.g., Ctrl+C) as exit
    }
}

fn select_server_in_category<'a>(
    all_ts_servers: &'a [Server],
    selected_category_name: &str,
    config: &AppConfig,
) -> Result<ServerSelectionAction<'a>> {
    let allowed_names_in_category = config
        .allowed
        .get(selected_category_name)
        .cloned()
        .unwrap_or_else(Vec::new);

    let filtered_server_refs: Vec<&Server> = all_ts_servers
        .iter()
        .filter(|server| allowed_names_in_category.contains(&server.name))
        .collect();

    // Handle no configured/active servers for the category
    if allowed_names_in_category.is_empty() {
        println!(
            "No servers configured for category '{}'. Check config.json.",
            selected_category_name
        );
        return Ok(ServerSelectionAction::GoBack);
    }
    if filtered_server_refs.is_empty() {
        println!(
            "No active Tailscale servers found for '{}'. Check 'tailscale status'.",
            selected_category_name
        );
        return Ok(ServerSelectionAction::GoBack);
    }

    let mut display_options: Vec<Server> =
        filtered_server_refs.iter().map(|&s| s.clone()).collect();
    display_options.push(Server {
        name: "Go Back to Categories".to_string(),
        ip: String::new(),
    });
    display_options.push(Server {
        name: "Exit Application".to_string(),
        ip: String::new(),
    });

    let ans = Select::new("Select a server", display_options).prompt();

    match ans {
        Ok(chosen_server) => {
            if chosen_server.name == "Exit Application" {
                Ok(ServerSelectionAction::ExitApp)
            } else if chosen_server.name == "Go Back to Categories" {
                Ok(ServerSelectionAction::GoBack)
            } else {
                // Find the original server reference
                let actual_server = all_ts_servers
                    .iter()
                    .find(|s| s.name == chosen_server.name)
                    .ok_or_else(|| eyre!("Selected server not found internally."))?;
                Ok(ServerSelectionAction::SshInto(actual_server))
            }
        }
        Err(_) => Ok(ServerSelectionAction::GoBack),
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let all_ts_servers = tailscale::get_servers().map_err(|e| {
        eprintln!("Error fetching Tailscale servers: {}", e);
        eyre!("Failed to get Tailscale list. Ensure Tailscale is running.")
    })?;

    if all_ts_servers.is_empty() {
        println!("No Tailscale servers found. Check 'tailscale status' and config.");
        return Ok(());
    }

    let config = config_loader::use_config().map_err(|e| {
        eprintln!("Application startup failed: {}", e);
        eyre!("Failed to load config. Check config.json.")
    })?;

    'main_loop: loop {
        let args: Vec<String> = env::args().collect();
        let cli_category_input = args.get(1).map(|s| s.as_str());

        let selected_category_name = match select_category(&config, cli_category_input)? {
            Some(name) => name,
            None => break 'main_loop,
        };

        loop {
            // Step 2: Select a server within the chosen category
            match select_server_in_category(&all_ts_servers, &selected_category_name, &config)? {
                ServerSelectionAction::SshInto(server) => {
                    ssh_into_server(server, &config.user)?;
                    return Ok(());
                }
                ServerSelectionAction::GoBack => {
                    break;
                }
                ServerSelectionAction::ExitApp => {
                    break 'main_loop;
                }
            }
        }
    }

    Ok(())
}