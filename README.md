# korrect

A CLI tool to manage (and optionally fetch) multiple different versions of kubectl by providing a shim layer on top of kubectl. Based on the current k8s context, the correct kubectl version will automatically be invoked. Correct Kubetctl. Korrect.

Witness the miracle of never again needlessly being admonished: `version difference between client (1.31) and server (1.29) exceeds the supported minor version skew of +/-1`

## Features

- Easy installation with automatic setup of required directories and symlinks
- Optional automated download of kubectl, or keep it entirely manual.
- Shell completion support for various shells (bash, zsh, fish)
- Low performance overhead via caching based on kubeconfig content
- Supports common kubectl aliases (`k` and `kubectl`)
- Can uninstall itself: Satisfaction guaranteed or just call `korrect setup --uninstall`.

## Installation

### From Source

```bash
git clone https://gitlab.com/cromulentbanana/korrect.git
cd korrect
cargo install --path .
```

### Using Cargo

```bash
cargo install korrect
```

## Quick Start

1. Install korrect:
```bash
korrect setup
```

2. Add the korrect bin directory to your PATH:
```bash
export PATH="$HOME/.korrect/bin:$PATH"
```

3. Start using kubectl as normal - korrect will automatically intercept and process your commands.

## Usage

### Basic Commands

```bash
# Set up korrect
korrect setup

# List installed components
korrect list

# Generate shell completions (replace `zsh` with your shell)
korrect completions zsh
```

### Setup Options

The setup command supports several flags to customize installation:

```bash
# Force installation (overwrite existing files)
korrect setup --force

# Uninstall korrect
korrect setup --uninstall

# Auto-download latest components
korrect setup --auto-download
```


### Directory Structure

After installation, korrect creates the following directory structure:

```
~/
├──.cache/korrect/     # The contents of the active kubeconfig are hashed to reference
├                      # the version-specific kubectl belonging to it 
└──.korrect/bin/
    ├── k              # Symlink to kubectl-shim
    ├── kubectl        # Symlink to kubectl-shim
    ├── kubectl-shim   # The executable which dispatches specific kubectl versions
    ├── kubectl-vA.B.C # Binary auto-downloaded by kubectl-shim or placed here by you
    └── kubectl-vX.Y.Z # Binary auto-downloaded or kubectl-shim or placed here by you
```

## Configuration

Coming Soon: Configuration files can be placed in `~/.config/korrect/config.toml`. Example configuration:

```toml
[korrect]
key = value

[korrect-shim]
korrect_dir = ~/.some_other_dir
auto_download = true
```

## Shell Completion

korrect supports shell completions for various shells. To enable completions:

```bash
# For bash
korrect completions bash > ~/.local/share/bash-completion/completions/korrect

# For zsh
korrect completions zsh > ~/.zfunc/_korrect

# For fish
korrect completions fish > ~/.config/fish/completions/korrect.fish
```

Remember to source your shell's completion file or restart your shell.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## Acknowledgments

- Inspired by the volta.sh the Hassle-free javascript tool manager
- Built with [clap-rs](https://github.com/clap-rs/clap) for argument parsing, shell completion
