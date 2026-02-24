mod cli;
mod config;
mod editor;
mod layout;
mod model;
mod paths;
mod template;
pub mod zellij;

use anyhow::Result;
use clap::Parser;
use std::process;

use crate::{cli::Cli, config::Config, editor::EditorApp};

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let _config = Config::load()?;
    let layout = template::load(&cli.name)?;

    if cli.dry_run {
        let text = template::save_dry_run(&layout);
        println!("{text}");
        return Ok(());
    }

    EditorApp::new(layout, cli.name).run()?;
    Ok(())
}
