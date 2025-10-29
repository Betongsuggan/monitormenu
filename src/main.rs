use clap::Parser;
use monitormenu::{
    backend::HyprlandBackend, cli::Cli, format_mode_for_display, format_monitor_for_display,
    parse_monitor_name_from_selection, Action,
};

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let launcher: monitormenu::launcher::Launcher = cli.launcher.into();
    let backend = HyprlandBackend::new();

    let monitors = backend.list_monitors()?;

    if monitors.is_empty() {
        eprintln!("No monitors found");
        return Ok(());
    }

    let monitor_options: Vec<String> = monitors
        .iter()
        .map(|m| format_monitor_for_display(m))
        .collect();

    let monitor_selection = launcher.show_menu(&monitor_options, Some("Select monitor:"))?;

    let monitor_name = parse_monitor_name_from_selection(&monitor_selection)
        .ok_or_else(|| anyhow::anyhow!("Failed to parse monitor name from selection"))?;

    let selected_monitor = monitors
        .iter()
        .find(|m| m.name == monitor_name)
        .ok_or_else(|| anyhow::anyhow!("Selected monitor not found"))?;

    let parsed_modes = HyprlandBackend::parse_modes(&selected_monitor.available_modes);

    let mut action_options: Vec<String> = parsed_modes
        .iter()
        .map(|mode| {
            format_mode_for_display(
                mode,
                selected_monitor.width,
                selected_monitor.height,
                selected_monitor.refresh_rate,
            )
        })
        .collect();

    if !selected_monitor.disabled {
        action_options.push("Disable monitor".to_string());
    } else {
        action_options.push("Enable monitor".to_string());
    }

    let action_selection = launcher.show_menu(
        &action_options,
        Some(&format!("Action for {}:", monitor_name)),
    )?;

    let action = Action::parse_from_selection(&action_selection)
        .ok_or_else(|| anyhow::anyhow!("Failed to parse action from selection"))?;

    match action {
        Action::SetMode(width, height, refresh) => {
            backend.set_monitor_mode(&monitor_name, width, height, refresh)?;
            println!(
                "Set monitor {} to {}x{}@{:.2}Hz",
                monitor_name, width, height, refresh
            );
        }
        Action::Enable => {
            backend.enable_monitor(&monitor_name)?;
            println!("Enabled monitor {}", monitor_name);
        }
        Action::Disable => {
            backend.disable_monitor(&monitor_name)?;
            println!("Disabled monitor {}", monitor_name);
        }
    }

    Ok(())
}
