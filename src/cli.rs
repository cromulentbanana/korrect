use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    author, 
    version = concat!(env!("CARGO_PKG_VERSION"), concat!("_", env!("GIT_SHORT_HASH"))),
    about,
    color = clap::ColorChoice::Auto,
    long_about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Setup korrect-shim in ~/.korrect/bin
    Setup,
    /// Generate shell completions
    Completions,
    /// List installed components
    List,
}
