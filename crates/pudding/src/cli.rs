use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "pudding", version, about = "Minimal pane multiplexer")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(about = "テンプレートを編集")]
    Template {
        #[command(subcommand)]
        command: TemplateCommand,
    },
    #[command(about = "テンプレートを適用して起動")]
    Run {
        #[arg(long, default_value = "default")]
        template: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum TemplateCommand {
    #[command(about = "AAエディタでテンプレートを編集")]
    Edit {
        #[arg(long, default_value = "default")]
        name: String,
    },
    #[command(about = "テンプレートを適用して起動")]
    Apply {
        #[arg(long, default_value = "default")]
        name: String,
    },
}
