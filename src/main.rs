use std::{io, process::Command};

use clap::Parser;

mod app;
mod cli;
mod host;
mod ui;

use app::App;
use cli::Cli;

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    if cli.version {
        print_versions()?;
        return Ok(());
    }

    if !cli.ssh_args.is_empty() {
        Command::new("ssh")
            .args(cli.ssh_args)
            .status()?;
        return Ok(());
    }

    let connect_target = ratatui::run(|terminal| App::default().run(terminal))?;

    if let Some(target) = connect_target {
        Command::new("ssh")
            .arg(target)
            .status()?;
    }

    Ok(())
}

fn print_versions() -> io::Result<()> {
    println!("KORA {}", env!("CARGO_PKG_VERSION"));

    let output = Command::new("ssh")
        .arg("-V")
        .output()?;
    
        let ssh_version = String::from_utf8_lossy(&output.stderr);

        println!("{}", ssh_version.trim());

        Ok(())
}