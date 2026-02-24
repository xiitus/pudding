use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "pudding", version, about = "zellij KDL layout editor")]
pub struct Cli {
    #[arg(long, default_value = "default")]
    pub name: String,
    #[arg(long)]
    pub dry_run: bool,
}
