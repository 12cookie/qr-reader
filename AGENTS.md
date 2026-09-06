# Agents Documentation

This document describes the automated agents and workflows used in the qr-reader project.

## Overview

The qr-reader project uses agents for continuous integration, code quality checks, and automated testing.

## Build & Test Agent

The build agent automatically compiles the project and runs tests on every pull request and push to main.

- **Trigger**: Every push and pull request
- **Commands**: 
  - `cargo build --release`
  - `cargo test --all`
- **Artifacts**: Compiled binaries in `target/release/`

## Code Quality Agent

The code quality agent checks for common issues and ensures code style consistency.

- **Trigger**: Every pull request
- **Checks**:
  - `cargo clippy -- -D warnings`
  - `cargo fmt -- --check`

## Documentation Agent

Validates that documentation is up-to-date and correctly formatted.

- **Trigger**: Pull requests affecting docs
- **Checks**: Markdown linting, broken links in README.md and other docs

## Contributing

When submitting changes that may be affected by agents:

1. Ensure your code passes `cargo clippy -- -D warnings`
2. Format your code with `cargo fmt`
3. Run all tests with `cargo test --all`
4. Update documentation if needed

For more information on the project structure and usage, see [README.md](README.md).
