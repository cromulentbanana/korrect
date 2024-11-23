{ pkgs, lib, config, inputs, ... }:

{
  # https://devenv.sh/basics/
  # https://devenv.sh/packages/
  packages = [
    pkgs.git
  ] ++ lib.optionals pkgs.stdenv.isDarwin [
    pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
  ];

  # https://devenv.sh/languages/
  languages.rust.enable = true;

  # https://devenv.sh/processes/
  processes.cargo-watch.exec = "cargo-watch";

  # https://devenv.sh/scripts/
  scripts.install.exec = ''
    cargo install --path .
  '';

  enterShell = ''
  '';

  # https://devenv.sh/tasks/
  # https://devenv.sh/tests/
  enterTest = ''
    # echo "Running tests"
    # git --version | grep --color=auto "${pkgs.git.version}"
  '';
  # See full reference at https://devenv.sh/reference/options/
}
