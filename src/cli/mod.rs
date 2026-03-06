use crate::backend::BackendType;
use crate::launcher::Launcher;
use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(name = "monitormenu")]
#[command(author, version, about, long_about = None)]
#[command(about = "Launcher-driven monitor manager for Wayland compositors")]
pub struct Cli {
    #[arg(short, long, value_enum, default_value = "walker")]
    pub launcher: CliLauncher,

    #[arg(short, long, value_enum, default_value = "auto")]
    pub backend: CliBackend,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum CliLauncher {
    Walker,
    Rofi,
    Dmenu,
    Fuzzel,
    Vicinae,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum CliBackend {
    Auto,
    Hyprland,
    Niri,
}

impl From<CliLauncher> for Launcher {
    fn from(cli: CliLauncher) -> Self {
        match cli {
            CliLauncher::Walker => Launcher::Walker,
            CliLauncher::Rofi => Launcher::Rofi,
            CliLauncher::Dmenu => Launcher::Dmenu,
            CliLauncher::Fuzzel => Launcher::Fuzzel,
            CliLauncher::Vicinae => Launcher::Vicinae,
        }
    }
}

impl From<CliBackend> for BackendType {
    fn from(cli: CliBackend) -> Self {
        match cli {
            CliBackend::Auto => BackendType::Auto,
            CliBackend::Hyprland => BackendType::Hyprland,
            CliBackend::Niri => BackendType::Niri,
        }
    }
}
