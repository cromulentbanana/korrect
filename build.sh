#!/bin/bash
set -euo pipefail

# Detect build environment
if [ -n "${CI:-}" ]; then
    echo "Running in GitLab CI environment"
elif [ -n "${DOCKER_BUILD:-}" ]; then
    echo "Running in Docker build environment"
else
    echo "This script is only intended to run in a CI/container environment"
    exit 0
fi

# Ensure required dependencies are installed
if command -v apt-get >/dev/null 2>&1; then
    apt-get update
    apt-get install -y musl-tools gcc-aarch64-linux-gnu libssl-dev
fi

# Set up Rust targets
rustup target add x86_64-unknown-linux-musl
rustup target add aarch64-unknown-linux-musl

# Set cross-compilation environment variables
export CC_aarch64_unknown_linux_musl=aarch64-linux-gnu-gcc
export AR_aarch64_unknown_linux_musl=aarch64-linux-gnu-ar
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-gnu-gcc

# Create output directory
mkdir -p dist

# Build for x86_64 Linux
cargo build --release --target x86_64-unknown-linux-musl
cp target/x86_64-unknown-linux-musl/release/korrect dist/korrect-x86_64-linux
cp target/x86_64-unknown-linux-musl/release/korrect-shim dist/korrect-shim-x86_64-linux

# Build for aarch64 Linux
cargo build --release --target aarch64-unknown-linux-musl
cp target/aarch64-unknown-linux-musl/release/korrect dist/korrect-aarch64-linux
cp target/aarch64-unknown-linux-musl/release/korrect-shim dist/korrect-shim-aarch64-linux

# Create archives
cd dist
tar -czf korrect-x86_64-linux.tar.gz korrect-x86_64-linux korrect-shim-x86_64-linux
tar -czf korrect-aarch64-linux.tar.gz korrect-aarch64-linux korrect-shim-aarch64-linux

# Generate checksums
sha256sum korrect-x86_64-linux.tar.gz > korrect-x86_64-linux.tar.gz.sha256
sha256sum korrect-aarch64-linux.tar.gz > korrect-aarch64-linux.tar.gz.sha256
