build:
    cargo build

test:
    cargo test

fmt:
    pre-commit run --all-files

check:
    cargo check
    just clippy
    cargo fmt --all -- --check
    cargo doc --all-features --no-deps

clippy:
    cargo clippy --all-targets --all-features -- \
      -D warnings \
      -D clippy::all \
      -D clippy::pedantic \
      -D clippy::cargo \
      -A clippy::multiple_crate_versions

release:
    cargo build --release
    cp -p ./target/release/ink ~/bin/ink

install target="debug":
    cp ./target/{{target}}/ink ~/bin/ink

debug cmd="view":
    ./target/debug/ink {{cmd}}

run *args:
    cargo run {{args}}

# gh

pr:
    gh pr create --assignee=@me
