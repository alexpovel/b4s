[private]
default:
    @just --list --justfile {{justfile()}}

set shell := ["bash", "-c"]
set windows-shell := ["pwsh.exe", "-NoLogo", "-Command"]
set ignore-comments := true

# Runs onboarding steps, installing dependencies and setting up the environment.
onboard: install-pre-commit install-afl
    pre-commit install --hook-type pre-push --hook-type pre-commit --hook-type commit-msg
    cargo install cargo-tarpaulin

# Installs pre-commit, the program.
[unix]
install-pre-commit:
    # From Debian 12, `pip install` globally is an error ("This environment is externally managed.")
    command -v pre-commit > /dev/null || { pip install pre-commit || sudo apt-get update && sudo apt-get install --yes pre-commit; }

# Runs fuzz testing.
[unix]
fuzz: install-afl
    cd {{ justfile_directory() }} && cargo afl build --package afl-target

    # Kill any running fuzzers, they like to get stuck.
    cd {{ justfile_directory() }} && kill -9 $(lsof -t 'fuzz/out'/*) || true

    cd {{ justfile_directory() }} && cargo afl fuzz -i fuzz/in -o fuzz/out target/debug/afl-target

# Provisions AFL (https://rust-fuzz.github.io/book/afl/setup.html#tools).
[unix]
install-afl:
    command -v gcc > /dev/null || { sudo apt-get update && sudo apt-get install --yes gcc; }
    command -v make > /dev/null || { sudo apt-get update && sudo apt-get install --yes make; }
    command -v lsof > /dev/null || { sudo apt-get update && sudo apt-get install --yes lsof; }
    command -v cargo-afl > /dev/null || { cargo install afl; }
