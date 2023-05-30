[private]
default:
    @just --list --justfile {{justfile()}}

set shell := ["bash", "-c"]
set windows-shell := ["pwsh.exe", "-NoLogo", "-Command"]
set ignore-comments := true

# Runs onboarding steps, installing dependencies and setting up the environment.
onboard:
    pip install pre-commit && pre-commit install --hook-type pre-push --hook-type pre-commit --hook-type commit-msg
    cargo install cargo-tarpaulin
