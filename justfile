build:
    cargo build

release:
    cargo build --release
    cp -p ./target/release/ink ~/bin/ink

install target="debug":
    cp ./target/{{target}}/ink ~/bin/ink

test cmd="view":
    ./target/debug/ink {{cmd}}
