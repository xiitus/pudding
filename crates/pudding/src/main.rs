mod action;
mod cli;
mod config;
mod editor;
mod keybind;
mod layout;
mod model;
mod paths;
mod runtime;
mod template;

use anyhow::Result;
use clap::Parser;

use crate::{
    cli::{Cli, Command, TemplateCommand},
    config::Config,
    editor::EditorApp,
    runtime::RuntimeApp,
    template::load_template,
};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::load();

    match cli.command {
        None => run_template("default", &config),
        Some(Command::Run { template }) => run_template(&template, &config),
        Some(Command::Template { command }) => match command {
            TemplateCommand::Edit { name } => {
                let mut template = load_template(&name)?;
                template.name = name;
                let _ = EditorApp::new(template).run()?;
                Ok(())
            }
            TemplateCommand::Apply { name } => run_template(&name, &config),
        },
    }
}

fn run_template(name: &str, config: &Config) -> Result<()> {
    let mut template = load_template(name)?;
    template.name = name.to_string();
    let app = RuntimeApp::new(template, config.clone())?;
    app.run()
}
