use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
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