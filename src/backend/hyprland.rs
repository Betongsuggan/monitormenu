use anyhow::{Context, Result};
use serde::Deserialize;
use std::process::Command;

use super::common::{Backend, Capabilities, Mode, Monitor, OutputConfig};
use crate::fmt_num;

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
    pub scale: f32,
    pub transform: i32,
    pub focused: bool,
    pub vrr: bool,
    pub disabled: bool,
    #[serde(rename = "availableModes")]
    pub available_modes: Vec<String>,
    #[serde(rename = "currentFormat", default)]
    pub current_format: String,
    /// "none", or the id (older releases: the name) of the mirrored output
    #[serde(rename = "mirrorOf", default)]
    pub mirror_of: String,
    #[serde(rename = "colorManagementPreset", default)]
    pub color_management_preset: String,
    #[serde(rename = "sdrBrightness", default)]
    pub sdr_brightness: Option<f32>,
    #[serde(rename = "sdrSaturation", default)]
    pub sdr_saturation: Option<f32>,
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

        let stdout = String::from_utf8(output.stdout)?;
        // hyprctl reports a rejected keyword on stdout with exit status 0
        if !output.status.success() || stdout.trim_start().starts_with("error") {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("hyprctl failed: {}{}", stdout.trim(), stderr.trim());
        }

        Ok(stdout)
    }
}

pub fn parse_modes(mode_strings: &[String]) -> Vec<Mode> {
    mode_strings
        .iter()
        .filter_map(|s| {
            let (res, refresh) = s.split_once('@')?;
            let (width, height) = res.split_once('x')?;
            Some(Mode {
                width: width.parse().ok()?,
                height: height.parse().ok()?,
                refresh_rate: refresh.trim_end_matches("Hz").parse().ok()?,
            })
        })
        .collect()
}

/// `hyprctl monitors all -j` as monitors
pub fn parse_monitors(json: &str) -> Result<Vec<Monitor>> {
    let hypr: Vec<HyprlandMonitor> =
        serde_json::from_str(json).context("Failed to parse monitor list from hyprctl")?;

    let name_of = |mirror: &str| -> Option<String> {
        if mirror.is_empty() || mirror == "none" {
            return None;
        }
        match mirror.parse::<i32>() {
            Ok(id) => hypr.iter().find(|m| m.id == id).map(|m| m.name.clone()),
            Err(_) => Some(mirror.to_string()),
        }
    };

    Ok(hypr
        .iter()
        .map(|m| {
            let hdr = m.color_management_preset.starts_with("hdr");
            // Unity values are Hyprland's defaults, not settings
            let non_default = |v: Option<f32>| v.filter(|v| (v - 1.0).abs() > 0.001);
            Monitor {
                name: m.name.clone(),
                description: m.description.clone(),
                focused: m.focused,
                config: OutputConfig {
                    enabled: !m.disabled,
                    mode: Mode {
                        width: m.width,
                        height: m.height,
                        refresh_rate: m.refresh_rate,
                    },
                    x: m.x,
                    y: m.y,
                    scale: m.scale,
                    transform: m.transform.clamp(0, 7) as u8,
                    vrr: m.vrr,
                    hdr,
                    bitdepth: m.current_format.contains("2101010").then_some(10),
                    sdr_brightness: non_default(m.sdr_brightness),
                    sdr_saturation: non_default(m.sdr_saturation),
                    mirror_of: name_of(&m.mirror_of),
                },
                available_modes: parse_modes(&m.available_modes),
            }
        })
        .collect())
}

/// A whole `monitor` rule for `config`, in the syntax of Hyprland's config
/// (and of nix-home's my.window-manager.monitors rendering)
pub fn monitor_rule(name: &str, config: &OutputConfig) -> String {
    if !config.enabled {
        return format!("{},disable", name);
    }

    let c = config;
    let mut parts = vec![
        name.to_string(),
        format!(
            "{}x{}@{}",
            c.mode.width,
            c.mode.height,
            fmt_num(c.mode.refresh_rate)
        ),
        format!("{}x{}", c.x, c.y),
        fmt_num(c.scale),
    ];
    let mut push = |key: &str, value: String| {
        parts.push(key.to_string());
        parts.push(value);
    };

    if c.transform != 0 {
        push("transform", c.transform.to_string());
    }
    if let Some(mirror) = &c.mirror_of {
        push("mirror", mirror.clone());
    }
    // Explicit either way: a rule without it falls back to misc:vrr
    push("vrr", if c.vrr { "1" } else { "0" }.to_string());
    if c.hdr {
        push("bitdepth", c.bitdepth.unwrap_or(10).to_string());
        push("cm", "hdr".to_string());
        if let Some(v) = c.sdr_brightness {
            push("sdrbrightness", fmt_num(v));
        }
        if let Some(v) = c.sdr_saturation {
            push("sdrsaturation", fmt_num(v));
        }
    } else {
        if let Some(depth) = c.bitdepth {
            push("bitdepth", depth.to_string());
        }
        push("cm", "srgb".to_string());
    }

    parts.join(",")
}

impl Default for HyprlandBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl Backend for HyprlandBackend {
    fn list_monitors(&self) -> Result<Vec<Monitor>> {
        parse_monitors(&self.run_hyprctl(&["monitors", "all", "-j"])?)
    }

    fn apply(&self, monitor: &Monitor, config: &OutputConfig) -> Result<()> {
        // A disabled output reports no usable mode: switch it on at its
        // highest resolution and refresh rate, then it can be adjusted
        let rule = if config.enabled && !monitor.config.enabled {
            format!("{},highres,auto,1", monitor.name)
        } else {
            monitor_rule(&monitor.name, config)
        };

        self.run_hyprctl(&["keyword", "monitor", &rule])
            .with_context(|| format!("Failed to apply monitor rule {}", rule))?;
        Ok(())
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            hdr: true,
            mirror: true,
        }
    }

    fn name(&self) -> &'static str {
        "hyprland"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EDP: &str = r#"[{
        "id": 0, "name": "eDP-1", "description": "Lenovo Group Limited 0x41B5",
        "width": 1920, "height": 1200, "refreshRate": 60.00200, "x": 0, "y": 0,
        "scale": 1.00, "transform": 0, "focused": true, "vrr": false, "disabled": false,
        "currentFormat": "XRGB8888", "mirrorOf": "none", "colorManagementPreset": "srgb",
        "sdrBrightness": 1.00, "sdrSaturation": 1.00,
        "availableModes": ["1920x1200@60.00Hz","1920x1080@60.00Hz"]
    }, {
        "id": 1, "name": "DP-2", "description": "Some TV",
        "width": 3840, "height": 2160, "refreshRate": 119.88, "x": 1920, "y": 0,
        "scale": 1.50, "transform": 1, "focused": false, "vrr": true, "disabled": false,
        "currentFormat": "XRGB2101010", "mirrorOf": "0", "colorManagementPreset": "hdr",
        "sdrBrightness": 1.20, "sdrSaturation": 1.00,
        "availableModes": ["3840x2160@119.88Hz"]
    }]"#;

    #[test]
    fn parses_state() {
        let monitors = parse_monitors(EDP).unwrap();
        let edp = &monitors[0].config;
        assert!(edp.enabled && !edp.hdr && !edp.vrr);
        assert_eq!(edp.bitdepth, None);
        assert_eq!(edp.sdr_brightness, None);
        assert_eq!(edp.mirror_of, None);
        assert_eq!(monitors[0].available_modes.len(), 2);

        let tv = &monitors[1].config;
        assert!(tv.hdr && tv.vrr);
        assert_eq!(tv.bitdepth, Some(10));
        assert_eq!(tv.transform, 1);
        assert_eq!(tv.sdr_brightness, Some(1.2));
        assert_eq!(tv.mirror_of.as_deref(), Some("eDP-1"));
    }

    #[test]
    fn renders_full_rules() {
        let monitors = parse_monitors(EDP).unwrap();
        assert_eq!(
            monitor_rule("eDP-1", &monitors[0].config),
            "eDP-1,1920x1200@60.002,0x0,1,vrr,0,cm,srgb"
        );
        assert_eq!(
            monitor_rule("DP-2", &monitors[1].config),
            "DP-2,3840x2160@119.88,1920x0,1.5,transform,1,mirror,eDP-1,vrr,1,bitdepth,10,cm,hdr,sdrbrightness,1.2"
        );

        let mut off = monitors[0].config.clone();
        off.enabled = false;
        assert_eq!(monitor_rule("eDP-1", &off), "eDP-1,disable");
    }
}
