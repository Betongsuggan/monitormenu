use anyhow::{Context, Result};
use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Copy)]
pub enum Launcher {
    Walker,
    Rofi,
    Dmenu,
    Fuzzel,
}

impl Launcher {
    fn get_command(&self, prompt: Option<&str>) -> (String, Vec<String>) {
        match self {
            Launcher::Walker => {
                let mut args = vec!["-d".to_string()];
                if let Some(p) = prompt {
                    args.push("-p".to_string());
                    args.push(p.to_string());
                }
                ("walker".to_string(), args)
            }
            Launcher::Rofi => {
                let mut args = vec!["-dmenu".to_string()];
                if let Some(p) = prompt {
                    args.push("-p".to_string());
                    args.push(p.to_string());
                }
                ("rofi".to_string(), args)
            }
            Launcher::Dmenu => {
                let mut args = vec![];
                if let Some(p) = prompt {
                    args.push("-p".to_string());
                    args.push(p.to_string());
                }
                ("dmenu".to_string(), args)
            }
            Launcher::Fuzzel => {
                let mut args = vec!["--dmenu".to_string()];
                if let Some(p) = prompt {
                    args.push("--prompt".to_string());
                    args.push(p.to_string());
                }
                ("fuzzel".to_string(), args)
            }
        }
    }

    pub fn show_menu(&self, options: &[String], prompt: Option<&str>) -> Result<String> {
        let (cmd, args) = self.get_command(prompt);

        let mut child = Command::new(&cmd)
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .context(format!("Failed to spawn {}. Is it installed?", cmd))?;

        if let Some(mut stdin) = child.stdin.take() {
            for option in options {
                writeln!(stdin, "{}", option)?;
            }
        }

        let output = child.wait_with_output()?;

        if !output.status.success() {
            anyhow::bail!("Launcher exited with error");
        }

        let selection = String::from_utf8(output.stdout)?.trim().to_string();

        if selection.is_empty() {
            anyhow::bail!("No selection made");
        }

        Ok(selection)
    }
}

impl std::str::FromStr for Launcher {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "walker" => Ok(Launcher::Walker),
            "rofi" => Ok(Launcher::Rofi),
            "dmenu" => Ok(Launcher::Dmenu),
            "fuzzel" => Ok(Launcher::Fuzzel),
            _ => Err(anyhow::anyhow!("Unknown launcher: {}", s)),
        }
    }
}

impl std::fmt::Display for Launcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Launcher::Walker => write!(f, "walker"),
            Launcher::Rofi => write!(f, "rofi"),
            Launcher::Dmenu => write!(f, "dmenu"),
            Launcher::Fuzzel => write!(f, "fuzzel"),
        }
    }
}
