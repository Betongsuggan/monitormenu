use anyhow::{Context, Result};
use std::io::Write;
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

use crate::backend::{Backend, Monitor, OutputConfig};
use crate::launcher::Launcher;
use crate::{
    describe_placement, fmt_num, format_mode, format_monitor_for_display, place, rotation_label,
    to_nix, Side,
};

const SCALES: [f32; 5] = [1.0, 1.25, 1.5, 1.6, 2.0];

/// The settings menu's entries
enum Setting {
    Mode,
    Scale,
    Rotation,
    Placement,
    Vrr,
    Hdr,
    Enable,
    Disable,
    ShowNix,
}

fn marked(current: bool, label: String) -> String {
    format!("{} {}", if current { "●" } else { " " }, label)
}

fn on_off(v: bool) -> &'static str {
    if v {
        "on"
    } else {
        "off"
    }
}

/// Pick a monitor, then change its settings one at a time; the settings menu
/// comes back after each change, with the new state, until dismissed
pub fn run(launcher: Launcher, backend: &dyn Backend) -> Result<()> {
    let monitors = backend.list_monitors()?;
    if monitors.is_empty() {
        eprintln!("No monitors found");
        return Ok(());
    }

    let labels: Vec<String> = monitors.iter().map(format_monitor_for_display).collect();
    let Some(i) = launcher.choose(&labels, Some("Select monitor:"))? else {
        return Ok(());
    };
    let name = monitors[i].name.clone();

    loop {
        let monitors = backend.list_monitors()?;
        let Some(monitor) = monitors.iter().find(|m| m.name == name) else {
            anyhow::bail!("Monitor {} is gone", name);
        };
        let others: Vec<&Monitor> = monitors
            .iter()
            .filter(|m| m.name != name && m.config.enabled && m.config.mirror_of.is_none())
            .collect();

        let Some(new) = settings_menu(launcher, backend, monitor, &others)? else {
            return Ok(());
        };
        if new != monitor.config {
            backend.apply(monitor, &new)?;
            // Let the compositor settle before reading the state back
            sleep(Duration::from_millis(300));
        }
    }
}

/// One round of the settings menu: the new config to apply (unchanged when
/// nothing is to be applied), or None when the menu is dismissed
fn settings_menu(
    launcher: Launcher,
    backend: &dyn Backend,
    monitor: &Monitor,
    others: &[&Monitor],
) -> Result<Option<OutputConfig>> {
    let c = &monitor.config;
    let caps = backend.capabilities();

    let mut entries: Vec<(String, Setting)> = Vec::new();
    if c.enabled {
        entries.push((
            format!("Resolution: {}  ▸", format_mode(&c.mode)),
            Setting::Mode,
        ));
        entries.push((format!("Scale: {}  ▸", fmt_num(c.scale)), Setting::Scale));
        entries.push((
            format!("Rotation: {}  ▸", rotation_label(c.transform)),
            Setting::Rotation,
        ));
        if !others.is_empty() {
            entries.push((
                format!("Placement: {}  ▸", describe_placement(monitor, others)),
                Setting::Placement,
            ));
        }
        entries.push((
            format!("VRR: {} (select to turn {})", on_off(c.vrr), on_off(!c.vrr)),
            Setting::Vrr,
        ));
        if caps.hdr {
            entries.push((
                format!("HDR: {} (select to turn {})", on_off(c.hdr), on_off(!c.hdr)),
                Setting::Hdr,
            ));
        }
        // Never switch off the last lit output
        if !others.is_empty() {
            entries.push(("Disable monitor".to_string(), Setting::Disable));
        }
    } else {
        entries.push(("Enable monitor".to_string(), Setting::Enable));
    }
    entries.push(("Show as Nix (copies it)".to_string(), Setting::ShowNix));

    let labels: Vec<String> = entries.iter().map(|(l, _)| l.clone()).collect();
    let prompt = format!("{}:", monitor.name);
    let Some(i) = launcher.choose(&labels, Some(&prompt))? else {
        return Ok(None);
    };

    let mut new = c.clone();
    match entries[i].1 {
        Setting::Mode => {
            let labels: Vec<String> = monitor
                .available_modes
                .iter()
                .map(|m| {
                    let current = m.width == c.mode.width
                        && m.height == c.mode.height
                        && (m.refresh_rate - c.mode.refresh_rate).abs() < 0.01;
                    marked(current, format_mode(m))
                })
                .collect();
            if let Some(j) = launcher.choose(&labels, Some("Resolution:"))? {
                new.mode = monitor.available_modes[j].clone();
            }
        }
        Setting::Scale => {
            let labels: Vec<String> = SCALES
                .iter()
                .map(|s| marked((s - c.scale).abs() < 0.01, fmt_num(*s)))
                .collect();
            if let Some(j) = launcher.choose(&labels, Some("Scale:"))? {
                new.scale = SCALES[j];
            }
        }
        Setting::Rotation => {
            let labels: Vec<String> = (0..4)
                .map(|t| marked(t == c.transform, rotation_label(t).to_string()))
                .collect();
            if let Some(j) = launcher.choose(&labels, Some("Rotation:"))? {
                new.transform = j as u8;
            }
        }
        Setting::Placement => {
            // (label, position, mirror)
            let mut choices: Vec<(String, (i32, i32), Option<String>)> = Vec::new();
            for neighbor in others {
                for side in Side::ALL {
                    let at = place(monitor, neighbor, side);
                    let current = c.mirror_of.is_none() && at == (c.x, c.y);
                    let label = format!(
                        "Extend {} {} ({})",
                        side.label(),
                        neighbor.name,
                        neighbor.description
                    );
                    choices.push((marked(current, label), at, None));
                }
            }
            if caps.mirror {
                for neighbor in others {
                    let current = c.mirror_of.as_deref() == Some(neighbor.name.as_str());
                    let label = format!("Mirror {} ({})", neighbor.name, neighbor.description);
                    choices.push((
                        marked(current, label),
                        (c.x, c.y),
                        Some(neighbor.name.clone()),
                    ));
                }
            }
            let labels: Vec<String> = choices.iter().map(|(l, _, _)| l.clone()).collect();
            if let Some(j) = launcher.choose(&labels, Some("Placement:"))? {
                let (_, (x, y), mirror) = choices.swap_remove(j);
                new.x = x;
                new.y = y;
                new.mirror_of = mirror;
            }
        }
        Setting::Vrr => new.vrr = !c.vrr,
        Setting::Hdr => {
            new.hdr = !c.hdr;
            // HDR needs a 10-bit format; plain SDR goes back to the default
            new.bitdepth = if new.hdr { Some(10) } else { None };
        }
        Setting::Enable => new.enabled = true,
        Setting::Disable => new.enabled = false,
        Setting::ShowNix => show_nix(&to_nix(&monitor.name, c))?,
    }
    Ok(Some(new))
}

/// Copy the Nix to the clipboard and show it in a notification
fn show_nix(nix: &str) -> Result<()> {
    let mut copy = Command::new("wl-copy")
        .stdin(Stdio::piped())
        .spawn()
        .context("Failed to run wl-copy")?;
    if let Some(mut stdin) = copy.stdin.take() {
        stdin.write_all(nix.as_bytes())?;
    }
    copy.wait()?;

    Command::new("notify-send")
        .args(["Monitor config copied", nix])
        .status()
        .context("Failed to run notify-send")?;
    println!("{}", nix);
    Ok(())
}
