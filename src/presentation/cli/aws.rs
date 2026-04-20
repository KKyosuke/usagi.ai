use anyhow::{Context, Result};
use directories::UserDirs;
use inquire::{Select, Text};
use std::collections::HashSet;
use std::fs;
use std::process::{Command, Stdio};

pub fn run_login(profile: Option<String>) -> Result<()> {
    // 1. Verify aws CLI is installed
    if Command::new("aws")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_err()
    {
        eprintln!("Error: aws CLI not found in PATH. Install/configure AWS CLI v2 first.");
        std::process::exit(2);
    }

    // 2. Extact profiles from ~/.aws/config and ~/.aws/credentials
    let unique_profiles = get_aws_profiles();

    let chosen = if let Some(p) = profile {
        if !unique_profiles.contains(&p) {
            println!(
                "Warning: profile '{}' not found in ~/.aws/config or credentials. Script will still attempt to use it.",
                p
            );
        }
        p
    } else if unique_profiles.is_empty() {
        println!("No profiles found in ~/.aws/config or ~/.aws/credentials.");
        let p = Text::new("Enter the AWS profile name you want to use:").prompt()?;
        if p.trim().is_empty() {
            eprintln!("No profile provided; aborting.");
            std::process::exit(1);
        }
        p
    } else {
        println!("Select AWS profile to use:");
        const CUSTOM: &str = "Type custom profile";
        let mut options = unique_profiles.clone();
        options.push(CUSTOM.to_string());

        let ans = Select::new("Enter number (or select to type a custom profile):", options).prompt()?;
        if ans == CUSTOM {
            loop {
                let p = Text::new("Profile name:").prompt()?;
                if !p.trim().is_empty() {
                    break p;
                }
                println!("Empty profile — try again.");
            }
        } else {
            ans
        }
    };

    println!("Exported AWS_PROFILE={}", chosen);
    println!("Running: aws sso login --profile \"{}\"", chosen);

    let mut cmd = Command::new("aws");
    cmd.arg("sso").arg("login").arg("--profile").arg(&chosen);

    let status = cmd.status().context("Failed to execute aws cli")?;
    if !status.success() {
        eprintln!("aws sso login failed for profile '{}'.", chosen);
        std::process::exit(3);
    }

    println!("SSO login completed for profile '{}'.", chosen);
    Ok(())
}

fn get_aws_profiles() -> Vec<String> {
    let mut profiles = Vec::new();
    let mut seen = HashSet::new();

    if let Some(user_dirs) = UserDirs::new() {
        let home = user_dirs.home_dir();

        let config_path = home.join(".aws/config");
        if let Ok(content) = fs::read_to_string(config_path) {
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with("[profile ") && line.ends_with(']') {
                    let p = line[9..line.len() - 1].trim().to_string();
                    if !p.is_empty() && seen.insert(p.clone()) {
                        profiles.push(p);
                    }
                }
            }
        }

        let creds_path = home.join(".aws/credentials");
        if let Ok(content) = fs::read_to_string(creds_path) {
            for line in content.lines() {
                let line = line.trim();
                // Match lines like [default] or [tenpla]
                if line.starts_with('[') && line.ends_with(']') && !line.starts_with("[profile ") {
                    let p = line[1..line.len() - 1].trim().to_string();
                    if !p.is_empty() && seen.insert(p.clone()) {
                        profiles.push(p);
                    }
                }
            }
        }
    }

    profiles
}
