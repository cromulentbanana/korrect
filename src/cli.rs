use clap::{builder::styling, ArgGroup, CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use std::io::{stdout, Error};

#[derive(Parser)]
#[command(
    author,
    version = concat!(env!("CARGO_PKG_VERSION"), concat!("_", env!("GIT_SHORT_HASH"))),
    about,
    styles = styles(),
    color = clap::ColorChoice::Auto,
    long_about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    #[clap(about = "Generates shell completions")]
    Completions {
        #[arg(value_enum)]
        #[arg(help = "Shell to generate completions for")]
        shell: Option<Shell>,
    },
    // ... rest of the Commands enum remains the same ...
}

pub fn generate_completions(shell: Option<Shell>) -> Result<(), Error> {
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();
    match shell {
        Some(shell_type) => {
            if cmd.get_matches().get_flag("help") {
                print_shell_specific_instructions(&bin_name, shell_type);
            } else {
                generate(shell_type, &mut cmd, "korrect", &mut stdout());
            }
        }
        None => {
            println!("\nTo install completions:");
            print_installation_instructions(&bin_name);
        }
    }

    Ok(())
}

fn print_shell_specific_instructions(bin_name: &str, shell: Shell) {
    println!(
        "\nTo install completions for {}, follow these steps:\n",
        shell
    );

    match shell {
        Shell::Bash => {
            println!("Bash:");
            println!("  # Create completions directory");
            println!("  mkdir -p ~/.bash_completion.d");
            println!("  # Generate and save completions");
            println!(
                "  {} completions bash > ~/.bash_completion.d/{}.bash",
                bin_name, bin_name
            );
            println!("  # Add to ~/.bashrc (if not already present)");
            println!(
                "  echo 'source ~/.bash_completion.d/{}.bash' >> ~/.bashrc",
                bin_name
            );
            println!("  # Reload shell configuration");
            println!("  source ~/.bashrc");
        }
        Shell::Zsh => {
            println!("Zsh:");
            println!("  # Create completions directory");
            println!("  mkdir -p ~/.zsh/completion");
            println!("  # Generate and save completions");
            println!(
                "  {} completions zsh > ~/.zsh/completion/_{}",
                bin_name, bin_name
            );
            println!("  # Add to ~/.zshrc (if not already present)");
            println!("  echo 'fpath=(~/.zsh/completion $fpath)' >> ~/.zshrc");
            println!("  echo 'autoload -U compinit; compinit' >> ~/.zshrc");
            println!("  # Reload shell configuration");
            println!("  source ~/.zshrc");
        }
        Shell::Fish => {
            println!("Fish:");
            println!("  # Create completions directory");
            println!("  mkdir -p ~/.config/fish/completions");
            println!("  # Generate and save completions");
            println!(
                "  {} completions fish > ~/.config/fish/completions/{}.fish",
                bin_name, bin_name
            );
            println!("  # Completions will be automatically loaded on next shell start");
        }
        Shell::Elvish => {
            println!("Elvish:");
            println!("  # Create completions directory");
            println!("  mkdir -p ~/.elvish/lib");
            println!("  # Generate and save completions");
            println!(
                "  {} completions elvish > ~/.elvish/lib/{}-completions.elv",
                bin_name, bin_name
            );
            println!("  # Add to ~/.elvish/rc.elv (if not already present)");
            println!("  echo 'use {}-completions' >> ~/.elvish/rc.elv", bin_name);
            println!("  # Restart Elvish or reload configuration");
        }
        _ => {
            println!("Shell-specific instructions not available for this shell.");
            println!(
                "You can generate completions to stdout and redirect them wherever you prefer:"
            );
            println!("  {} completions <shell>", bin_name);
        }
    }
}

fn print_installation_instructions(bin_name: &str) {
    println!("\nTo install completions, run one of the following commands based on your shell:\n");

    // Print instructions for each supported shell
    print_shell_specific_instructions(bin_name, Shell::Bash);
    println!("");
    print_shell_specific_instructions(bin_name, Shell::Zsh);
    println!("");
    print_shell_specific_instructions(bin_name, Shell::Fish);
    println!("");
    print_shell_specific_instructions(bin_name, Shell::Elvish);

    println!("\nNote: You can also generate completions to stdout and redirect them wherever you prefer:");
    println!("  {} completions <shell>", bin_name);
}

fn styles() -> styling::Styles {
    styling::Styles::plain()
        .header(
            styling::AnsiColor::BrightBlue.on_default()
                | styling::Effects::BOLD
                | styling::Effects::ITALIC,
        )
        .usage(
            styling::AnsiColor::BrightBlue.on_default()
                | styling::Effects::BOLD
                | styling::Effects::ITALIC,
        )
        .literal(styling::AnsiColor::Green.on_default() | styling::Effects::BOLD)
        .placeholder(styling::AnsiColor::BrightBlue.on_default())
}
