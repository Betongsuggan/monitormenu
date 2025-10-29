use crate::launcher::Launcher;
use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(name = "monitormenu")]
#[command(author, version, about, long_about = None)]
#[command(about = "Launcher-driven monitor manager for Hyprland")]
pub struct Cli {
    #[arg(short, long, value_enum, default_value = "walker")]
    pub launcher: CliLauncher,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum CliLauncher {
    Walker,
    Rofi,
    Dmenu,
    Fuzzel,
}

impl From<CliLauncher> for Launcher {
    fn from(cli: CliLauncher) -> Self {
        match cli {
            CliLauncher::Walker => Launcher::Walker,
            CliLauncher::Rofi => Launcher::Rofi,
            CliLauncher::Dmenu => Launcher::Dmenu,
            CliLauncher::Fuzzel => Launcher::Fuzzel,
        }
    }
}
