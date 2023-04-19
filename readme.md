# Celcius

A rust CLI to clean up your git repos

## Deving

Run with `cargo run`

## Deving CLI

`cargo run -- --verbosity prstats`

## Build a release
`cargo build --release`

## Using the .exe
`./celcius.exe prstats --token=<your-token>`

# Environment
GITHUB_API_TOKEN needs to be provided, either via argument when calling the CLI or via environment variable `GITHUB_API_TOKEN`.
The token needs to have several permissions:
- read:public_repo
- read:user
- read:org
- read:enterprise
