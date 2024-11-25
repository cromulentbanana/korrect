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

#[derive(Subcommand, Clone, Copy)]
pub enum Commands {
    #[clap(about = "Generates shell completions")]
    #[command(arg_required_else_help = true)]
    Completions {
        #[arg(value_enum)]
        #[arg(help = "Generates shell completion scripts")]
        shell: Option<Shell>,

        #[arg(long, short, hide = true)]
        help: bool,
    },
    #[clap(group(ArgGroup::new("exclusive_flags")
        .args(&["force", "uninstall"])
        .required(false)))]
    #[clap(about = "Installs the korrect-shim and creates the cache")]
    Setup {
        #[clap(long, default_value = "false")]
        #[clap(help = "Automatically download versions of kubectl when needed")]
        auto_download: bool,
        #[clap(long, default_value = "false", group = "exclusive_flags")]
        #[clap(help = "Overwrite existing korrect installed files")]
        force: bool,
        #[clap(long, default_value = "false", group = "exclusive_flags")]
        #[clap(help = "Remove all korrect installed files")]
        uninstall: bool,
    },
    #[clap(about = "Lists the installed components")]
    List,
}

pub fn generate_completions(shell: Option<Shell>, help: bool) -> Result<(), Error> {
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();
    match shell {
        Some(shell_type) => {
            if help {
                print_shell_specific_instructions(&bin_name, shell_type);
            } else {
                generate(shell_type, &mut cmd, "korrect", &mut stdout());
            }
        }
        None => {
            cmd.render_help();
            cmd.find_subcommand_mut("completions")
                .unwrap()
                .print_help()?;
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
