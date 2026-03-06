use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::process::Command;

use super::common::{Backend, Mode, Monitor};

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct NiriMode {
    width: u32,
    height: u32,
    refresh_rate: u32,
    is_preferred: bool,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct NiriLogical {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    scale: f64,
    transform: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct NiriOutput {
    name: String,
    make: String,
    model: String,
    serial: Option<String>,
    physical_size: Option<(u32, u32)>,
    modes: Vec<NiriMode>,
    current_mode: Option<usize>,
    vrr_supported: bool,
    vrr_enabled: bool,
    logical: Option<NiriLogical>,
}

pub struct NiriBackend;

impl NiriBackend {
    pub fn new() -> Self {
        Self
    }

    fn run_niri_msg(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("niri")
            .arg("msg")
            .args(args)
            .output()
            .context("Failed to execute niri msg. Is Niri running?")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("niri msg failed: {}", stderr);
        }

        Ok(String::from_utf8(output.stdout)?)
    }

    fn parse_modes(niri_modes: &[NiriMode]) -> Vec<Mode> {
        niri_modes
            .iter()
            .map(|m| Mode {
                width: m.width as i32,
                height: m.height as i32,
                refresh_rate: m.refresh_rate as f32 / 1000.0,
            })
            .collect()
    }
}

impl Default for NiriBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl Backend for NiriBackend {
    fn list_monitors(&self) -> Result<Vec<Monitor>> {
        let output = self.run_niri_msg(&["--json", "outputs"])?;
        let outputs: HashMap<String, NiriOutput> = serde_json::from_str(&output)
            .context("Failed to parse output list from niri msg")?;

        let monitors = outputs
            .into_iter()
            .map(|(connector, output)| {
                let description = format!("{} {}", output.make, output.model);

                let (width, height, refresh_rate) = output
                    .current_mode
                    .and_then(|idx| output.modes.get(idx))
                    .map(|m| (m.width as i32, m.height as i32, m.refresh_rate as f32 / 1000.0))
                    .unwrap_or((0, 0, 0.0));

                let (x, y, scale) = output
                    .logical
                    .as_ref()
                    .map(|l| (l.x, l.y, l.scale as f32))
                    .unwrap_or((0, 0, 1.0));

                let enabled = output.logical.is_some();
                let available_modes = Self::parse_modes(&output.modes);

                Monitor {
                    name: connector,
                    description,
                    width,
                    height,
                    refresh_rate,
                    x,
                    y,
                    scale,
                    focused: false,
                    enabled,
                    available_modes,
                }
            })
            .collect();

        Ok(monitors)
    }

    fn set_monitor_mode(&self, monitor: &str, width: i32, height: i32, refresh: f32) -> Result<()> {
        let mode_str = format!("{}x{}@{:.3}", width, height, refresh);

        self.run_niri_msg(&["output", monitor, "mode", &mode_str])
            .context("Failed to set monitor mode")?;

        Ok(())
    }

    fn enable_monitor(&self, monitor: &str) -> Result<()> {
        self.run_niri_msg(&["output", monitor, "on"])
            .context("Failed to enable monitor")?;
        Ok(())
    }

    fn disable_monitor(&self, monitor: &str) -> Result<()> {
        self.run_niri_msg(&["output", monitor, "off"])
            .context("Failed to disable monitor")?;
        Ok(())
    }

    fn name(&self) -> &'static str {
        "niri"
    }
}
