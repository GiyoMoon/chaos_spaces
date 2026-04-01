use clap::Subcommand;

#[derive(Clone, Subcommand)]
pub enum Commands {
    GenerateOpenapi,
}

#[derive(Clone, clap::Parser)]
pub struct Config {
    #[command(subcommand)]
    pub command: Option<Commands>,
    #[clap(long, env)]
    pub bind_address: String,
    #[clap(long, env)]
    pub database_url: String,
}
