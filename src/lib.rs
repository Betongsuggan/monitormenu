pub mod backend;
pub mod cli;
pub mod launcher;

use backend::{HyprlandMonitor, Mode};

#[derive(Debug, Clone)]
pub enum Action {
    SetMode(i32, i32, f32),
    Enable,
    Disable,
}

impl Action {
    pub fn parse_from_selection(selection: &str) -> Option<Self> {
        if selection.starts_with("Set: ") {
            let parts: Vec<&str> = selection
                .trim_start_matches("Set: ")
                .split(" @ ")
                .collect();
            if parts.len() == 2 {
                let res_parts: Vec<&str> = parts[0].split('x').collect();
                if res_parts.len() == 2 {
                    let width = res_parts[0].parse::<i32>().ok()?;
                    let height = res_parts[1].parse::<i32>().ok()?;
                    let refresh = parts[1].trim_end_matches(" Hz").parse::<f32>().ok()?;
                    return Some(Action::SetMode(width, height, refresh));
                }
            }
        } else if selection == "Enable monitor" {
            return Some(Action::Enable);
        } else if selection == "Disable monitor" {
            return Some(Action::Disable);
        }
        None
    }
}

pub fn format_monitor_for_display(monitor: &HyprlandMonitor) -> String {
    let status_icon = if monitor.dpms_status { "✓" } else { "✗" };
    let focused_icon = if monitor.focused { "●" } else { " " };

    format!(
        "{} {} {} - {}x{}@{:.2}Hz ({})",
        status_icon,
        focused_icon,
        monitor.description,
        monitor.width,
        monitor.height,
        monitor.refresh_rate,
        monitor.name
    )
}

pub fn format_mode_for_display(mode: &Mode, current_width: i32, current_height: i32, current_refresh: f32) -> String {
    let is_current = mode.width == current_width
        && mode.height == current_height
        && (mode.refresh_rate - current_refresh).abs() < 0.01;

    let marker = if is_current { "●" } else { " " };

    format!(
        "{} Set: {}x{} @ {:.2} Hz",
        marker, mode.width, mode.height, mode.refresh_rate
    )
}

pub fn parse_monitor_name_from_selection(selection: &str) -> Option<String> {
    selection
        .split('(')
        .nth(1)?
        .split(')')
        .next()
        .map(|s| s.to_string())
}
