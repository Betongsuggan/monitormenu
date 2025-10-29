use anyhow::{Context, Result};
use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Deserialize, Clone)]
pub struct HyprlandMonitor {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub width: i32,
    pub height: i32,
    #[serde(rename = "refreshRate")]
    pub refresh_rate: f32,
    pub x: i32,
    pub y: i32,
    #[serde(rename = "activeWorkspace")]
    pub active_workspace: WorkspaceInfo,
    pub reserved: [i32; 4],
    pub scale: f32,
    pub transform: i32,
    pub focused: bool,
    #[serde(rename = "dpmsStatus")]
    pub dpms_status: bool,
    #[serde(rename = "vrr")]
    pub vrr: bool,
    #[serde(rename = "availableModes")]
    pub available_modes: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WorkspaceInfo {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Mode {
    pub width: i32,
    pub height: i32,
    pub refresh_rate: f32,
}

pub struct HyprlandBackend;

impl HyprlandBackend {
    pub fn new() -> Self {
        Self
    }

    fn run_hyprctl(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("hyprctl")
            .args(args)
            .output()
            .context("Failed to execute hyprctl. Is Hyprland running?")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("hyprctl failed: {}", stderr);
        }

        Ok(String::from_utf8(output.stdout)?)
    }

    pub fn list_monitors(&self) -> Result<Vec<HyprlandMonitor>> {
        let output = self.run_hyprctl(&["monitors", "-j"])?;
        let monitors: Vec<HyprlandMonitor> = serde_json::from_str(&output)
            .context("Failed to parse monitor list from hyprctl")?;
        Ok(monitors)
    }

    pub fn parse_modes(mode_strings: &[String]) -> Vec<Mode> {
        mode_strings
            .iter()
            .filter_map(|s| {
                let parts: Vec<&str> = s.split('@').collect();
                if parts.len() != 2 {
                    return None;
                }

                let res_parts: Vec<&str> = parts[0].split('x').collect();
                if res_parts.len() != 2 {
                    return None;
                }

                let width = res_parts[0].parse::<i32>().ok()?;
                let height = res_parts[1].parse::<i32>().ok()?;
                let refresh = parts[1].trim_end_matches("Hz").parse::<f32>().ok()?;

                Some(Mode {
                    width,
                    height,
                    refresh_rate: refresh,
                })
            })
            .collect()
    }

    pub fn set_monitor_mode(&self, monitor: &str, width: i32, height: i32, refresh: f32) -> Result<()> {
        let mode_str = format!("{}x{}@{:.2}", width, height, refresh);
        let monitor_config = format!("{},{},auto,1", monitor, mode_str);

        self.run_hyprctl(&["keyword", "monitor", &monitor_config])
            .context("Failed to set monitor resolution and refresh rate")?;

        Ok(())
    }

    pub fn enable_monitor(&self, monitor: &str) -> Result<()> {
        self.run_hyprctl(&["keyword", "monitor", &format!("{},preferred,auto,1", monitor)])
            .context("Failed to enable monitor")?;
        Ok(())
    }

    pub fn disable_monitor(&self, monitor: &str) -> Result<()> {
        self.run_hyprctl(&["keyword", "monitor", &format!("{},disable", monitor)])
            .context("Failed to disable monitor")?;
        Ok(())
    }
}

impl Default for HyprlandBackend {
    fn default() -> Self {
        Self::new()
    }
}
