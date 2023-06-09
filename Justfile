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

# Runs fuzz testing.
[unix]
fuzz: install-afl
    cd {{ justfile_directory() / 'fuzz' }} && cargo afl build

    # Kill any running fuzzers, they like to get stuck.
    kill -9 $(lsof -t {{ justfile_directory() / 'fuzz' / 'out' }}/*) || true

    cd {{ justfile_directory() / 'fuzz' }} && cargo afl fuzz -i in -o out target/debug/afl-target

# Provisions AFL (https://rust-fuzz.github.io/book/afl/setup.html#tools).
[unix]
install-afl:
    command -v gcc > /dev/null || { sudo apt-get update && sudo apt-get install --yes gcc; }
    command -v make > /dev/null || { sudo apt-get update && sudo apt-get install --yes make; }
    command -v lsof > /dev/null || { sudo apt-get update && sudo apt-get install --yes lsof; }
    command -v cargo-afl > /dev/null || { cargo install afl; }
