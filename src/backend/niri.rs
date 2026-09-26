use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::process::Command;

use super::common::{Backend, Capabilities, Mode, Monitor, OutputConfig};
use crate::fmt_num;

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

    fn output_msg(&self, name: &str, args: &[&str]) -> Result<()> {
        let mut all = vec!["output", name];
        all.extend_from_slice(args);
        self.run_niri_msg(&all)
            .with_context(|| format!("Failed to run niri msg {}", all.join(" ")))?;
        Ok(())
    }
}

fn to_mode(m: &NiriMode) -> Mode {
    Mode {
        width: m.width as i32,
        height: m.height as i32,
        refresh_rate: m.refresh_rate as f32 / 1000.0,
    }
}

/// niri's IPC transform (`Normal`, `_90`, `Flipped270`, ...) as a wl_output
/// transform number
fn parse_transform(s: &str) -> u8 {
    let rotation = if s.contains("270") {
        3
    } else if s.contains("180") {
        2
    } else if s.contains("90") {
        1
    } else {
        0
    };
    rotation + if s.starts_with("Flipped") { 4 } else { 0 }
}

/// A wl_output transform number as a `niri msg output transform` argument
fn transform_arg(t: u8) -> &'static str {
    [
        "normal",
        "90",
        "180",
        "270",
        "flipped",
        "flipped-90",
        "flipped-180",
        "flipped-270",
    ][(t % 8) as usize]
}

impl Default for NiriBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl Backend for NiriBackend {
    fn list_monitors(&self) -> Result<Vec<Monitor>> {
        let output = self.run_niri_msg(&["--json", "outputs"])?;
        let outputs: HashMap<String, NiriOutput> =
            serde_json::from_str(&output).context("Failed to parse output list from niri msg")?;

        let mut monitors: Vec<Monitor> = outputs
            .into_iter()
            .map(|(connector, output)| {
                let mode = output
                    .current_mode
                    .and_then(|idx| output.modes.get(idx))
                    .map(to_mode)
                    .unwrap_or(Mode {
                        width: 0,
                        height: 0,
                        refresh_rate: 0.0,
                    });
                let logical = output.logical.as_ref();

                Monitor {
                    name: connector,
                    description: format!("{} {}", output.make, output.model),
                    focused: false,
                    config: OutputConfig {
                        enabled: logical.is_some(),
                        mode,
                        x: logical.map_or(0, |l| l.x),
                        y: logical.map_or(0, |l| l.y),
                        scale: logical.map_or(1.0, |l| l.scale as f32),
                        transform: logical.map_or(0, |l| parse_transform(&l.transform)),
                        vrr: output.vrr_enabled,
                        hdr: false,
                        bitdepth: None,
                        sdr_brightness: None,
                        sdr_saturation: None,
                        mirror_of: None,
                    },
                    available_modes: output.modes.iter().map(to_mode).collect(),
                }
            })
            .collect();
        monitors.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(monitors)
    }

    // One `niri msg output` call per field that changed
    fn apply(&self, monitor: &Monitor, config: &OutputConfig) -> Result<()> {
        let (old, new, name) = (&monitor.config, config, monitor.name.as_str());

        if !new.enabled {
            return self.output_msg(name, &["off"]);
        }
        if !old.enabled {
            return self.output_msg(name, &["on"]);
        }
        if new.mode != old.mode {
            let mode = format!(
                "{}x{}@{:.3}",
                new.mode.width, new.mode.height, new.mode.refresh_rate
            );
            self.output_msg(name, &["mode", &mode])?;
        }
        if new.scale != old.scale {
            self.output_msg(name, &["scale", &fmt_num(new.scale)])?;
        }
        if new.transform != old.transform {
            self.output_msg(name, &["transform", transform_arg(new.transform)])?;
        }
        if (new.x, new.y) != (old.x, old.y) {
            let (x, y) = (new.x.to_string(), new.y.to_string());
            self.output_msg(name, &["position", "set", &x, &y])?;
        }
        if new.vrr != old.vrr {
            self.output_msg(name, &["vrr", if new.vrr { "on" } else { "off" }])?;
        }
        Ok(())
    }

    // niri has no HDR and no output mirroring
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            hdr: false,
            mirror: false,
        }
    }

    fn name(&self) -> &'static str {
        "niri"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transforms_round_trip() {
        for (ipc, n) in [("Normal", 0), ("_90", 1), ("_270", 3), ("Flipped180", 6)] {
            assert_eq!(parse_transform(ipc), n);
        }
        assert_eq!(transform_arg(1), "90");
        assert_eq!(transform_arg(7), "flipped-270");
    }
}
