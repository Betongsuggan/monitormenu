pub mod common;
pub mod hyprland;
pub mod niri;

pub use common::{Backend, Capabilities, Mode, Monitor, OutputConfig};
pub use hyprland::HyprlandBackend;
pub use niri::NiriBackend;

use anyhow::{bail, Result};
use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendType {
    Auto,
    Hyprland,
    Niri,
}

impl BackendType {
    pub fn detect() -> Option<BackendType> {
        if env::var("NIRI_SOCKET").is_ok() {
            Some(BackendType::Niri)
        } else if env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
            Some(BackendType::Hyprland)
        } else {
            None
        }
    }
}

pub fn create_backend(backend_type: BackendType) -> Result<Box<dyn Backend>> {
    match backend_type {
        BackendType::Auto => {
            if let Some(detected) = BackendType::detect() {
                create_backend(detected)
            } else {
                bail!("Could not auto-detect window manager. Please specify --backend hyprland or --backend niri")
            }
        }
        BackendType::Hyprland => Ok(Box::new(HyprlandBackend::new())),
        BackendType::Niri => Ok(Box::new(NiriBackend::new())),
    }
}
