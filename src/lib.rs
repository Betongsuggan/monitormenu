pub mod backend;
pub mod cli;
pub mod launcher;
pub mod menu;

use backend::{Mode, Monitor, OutputConfig};

/// A number without trailing zeros: 60.002, 1.25, 2
pub fn fmt_num(v: f32) -> String {
    let s = format!("{:.3}", v);
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

pub fn format_mode(mode: &Mode) -> String {
    format!(
        "{}x{} @ {} Hz",
        mode.width,
        mode.height,
        fmt_num(mode.refresh_rate)
    )
}

pub fn format_monitor_for_display(monitor: &Monitor) -> String {
    let status_icon = if monitor.config.enabled { "✓" } else { "✗" };
    let focused_icon = if monitor.focused { "●" } else { " " };
    let state = if monitor.config.enabled {
        format_mode(&monitor.config.mode)
    } else {
        "off".to_string()
    };

    format!(
        "{} {} {} - {} ({})",
        status_icon, focused_icon, monitor.description, state, monitor.name
    )
}

pub fn rotation_label(transform: u8) -> &'static str {
    [
        "normal",
        "90°",
        "180°",
        "270°",
        "flipped",
        "flipped 90°",
        "flipped 180°",
        "flipped 270°",
    ][(transform % 8) as usize]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Right,
    Left,
    Above,
    Below,
}

impl Side {
    pub const ALL: [Side; 4] = [Side::Right, Side::Left, Side::Above, Side::Below];

    pub fn label(self) -> &'static str {
        match self {
            Side::Right => "right of",
            Side::Left => "left of",
            Side::Above => "above",
            Side::Below => "below",
        }
    }
}

/// Where `target` goes to sit on `side` of `neighbor`, edges touching and
/// top/left edges aligned, in layout coordinates
pub fn place(target: &Monitor, neighbor: &Monitor, side: Side) -> (i32, i32) {
    let (tw, th) = target.logical_size();
    let (nw, nh) = neighbor.logical_size();
    let (nx, ny) = (neighbor.config.x, neighbor.config.y);
    match side {
        Side::Right => (nx + nw, ny),
        Side::Left => (nx - tw, ny),
        Side::Above => (nx, ny - th),
        Side::Below => (nx, ny + nh),
    }
}

/// How `target` sits among the other enabled monitors, for the menu
pub fn describe_placement(target: &Monitor, others: &[&Monitor]) -> String {
    if let Some(mirror) = &target.config.mirror_of {
        return format!("mirror of {}", mirror);
    }
    for neighbor in others {
        for side in Side::ALL {
            if place(target, neighbor, side) == (target.config.x, target.config.y) {
                return format!("{} {}", side.label(), neighbor.name);
            }
        }
    }
    format!("at {},{}", target.config.x, target.config.y)
}

/// The output as a my.window-manager.monitors entry (nix-home), leaving out
/// what equals the option defaults
pub fn to_nix(name: &str, c: &OutputConfig) -> String {
    let mut lines = vec![format!("my.window-manager.monitors.{} = {{", name)];
    if !c.enabled {
        lines.push("  enable = false;".to_string());
    } else {
        lines.push(format!(
            "  mode = {{ width = {}; height = {}; refresh = {}; }};",
            c.mode.width,
            c.mode.height,
            fmt_num(c.mode.refresh_rate)
        ));
        lines.push(format!("  position = {{ x = {}; y = {}; }};", c.x, c.y));
        if (c.scale - 1.0).abs() > 0.001 {
            lines.push(format!("  scale = {};", fmt_num(c.scale)));
        }
        if c.transform != 0 {
            lines.push(format!("  transform = {};", c.transform));
        }
        if let Some(mirror) = &c.mirror_of {
            lines.push(format!("  mirror = \"{}\";", mirror));
        }
        if c.vrr {
            lines.push("  vrr = true;".to_string());
        }
        if c.hdr {
            lines.push("  hdr = true;".to_string());
        }
        if let Some(depth) = c.bitdepth {
            lines.push(format!("  bitdepth = {};", depth));
        }
        if let Some(v) = c.sdr_brightness {
            lines.push(format!("  sdrBrightness = {};", fmt_num(v)));
        }
        if let Some(v) = c.sdr_saturation {
            lines.push(format!("  sdrSaturation = {};", fmt_num(v)));
        }
    }
    lines.push("};".to_string());
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn monitor(name: &str, w: i32, h: i32, x: i32, y: i32, scale: f32, transform: u8) -> Monitor {
        Monitor {
            name: name.to_string(),
            description: String::new(),
            focused: false,
            config: OutputConfig {
                enabled: true,
                mode: Mode {
                    width: w,
                    height: h,
                    refresh_rate: 60.0,
                },
                x,
                y,
                scale,
                transform,
                vrr: false,
                hdr: false,
                bitdepth: None,
                sdr_brightness: None,
                sdr_saturation: None,
                mirror_of: None,
            },
            available_modes: vec![],
        }
    }

    #[test]
    fn formats_numbers() {
        assert_eq!(fmt_num(60.0), "60");
        assert_eq!(fmt_num(59.94), "59.94");
        assert_eq!(fmt_num(1.25), "1.25");
        assert_eq!(fmt_num(60.002), "60.002");
    }

    #[test]
    fn places_against_scaled_and_rotated_neighbours() {
        // A 4K TV at scale 2 is 1920x1080 in the layout
        let tv = monitor("HDMI-A-1", 3840, 2160, 0, 0, 2.0, 0);
        let laptop = monitor("eDP-1", 1920, 1200, 0, 0, 1.25, 0);
        assert_eq!(place(&laptop, &tv, Side::Right), (1920, 0));
        assert_eq!(place(&laptop, &tv, Side::Below), (0, 1080));
        assert_eq!(place(&laptop, &tv, Side::Left), (-1536, 0));
        assert_eq!(place(&laptop, &tv, Side::Above), (0, -960));

        // Rotated 90°: width and height swap
        let portrait = monitor("DP-1", 2560, 1440, 100, 0, 1.0, 1);
        assert_eq!(place(&laptop, &portrait, Side::Right), (1540, 0));
    }

    #[test]
    fn describes_placement() {
        let tv = monitor("HDMI-A-1", 1920, 1080, 0, 0, 1.0, 0);
        let laptop = monitor("eDP-1", 1920, 1200, 1920, 0, 1.0, 0);
        assert_eq!(describe_placement(&laptop, &[&tv]), "right of HDMI-A-1");
        let loose = monitor("eDP-1", 1920, 1200, 5000, 7, 1.0, 0);
        assert_eq!(describe_placement(&loose, &[&tv]), "at 5000,7");
    }

    #[test]
    fn renders_nix() {
        let mut m = monitor("DP-2", 2560, 1440, 1920, 0, 1.25, 1);
        m.config.vrr = true;
        m.config.hdr = true;
        m.config.bitdepth = Some(10);
        assert_eq!(
            to_nix(&m.name, &m.config),
            "my.window-manager.monitors.DP-2 = {\n  mode = { width = 2560; height = 1440; refresh = 60; };\n  position = { x = 1920; y = 0; };\n  scale = 1.25;\n  transform = 1;\n  vrr = true;\n  hdr = true;\n  bitdepth = 10;\n};"
        );
        m.config.enabled = false;
        assert_eq!(
            to_nix(&m.name, &m.config),
            "my.window-manager.monitors.DP-2 = {\n  enable = false;\n};"
        );
    }
}
