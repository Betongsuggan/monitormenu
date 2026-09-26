use clap::Parser;
use monitormenu::{backend::create_backend, cli::Cli, menu};

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let backend = create_backend(cli.backend.into())?;
    menu::run(cli.launcher.into(), backend.as_ref())
}
