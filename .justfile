list:
    just --list

# Test that all targets build correctly and tests pass
test:
    cargo build --release # Native build
    cargo geng build --platform web --release # Web build
    cargo test --all-features # Tests
    cargo check -F steam
    cargo check -F steam -F demo
    cargo check -F itch -F demo

game *ARGS:
    cargo run -- {{ARGS}}

web command *ARGS:
    cargo geng {{command}} --platform web --release -- {{ARGS}}

itch command *ARGS:
    cargo geng {{command}} --platform web --release -- -F itch {{ARGS}}

server PORT *ARGS:
    cargo run --release --package ctl-server {{PORT}} -- {{ARGS}}

build-server:
    docker build . --tag ctl-server
    docker run --name ctl-server --rm -it -e CARGO_TARGET_DIR=/target -v `pwd`/docker-target:/target -v `pwd`:/src -w /src ctl-server cargo build --release --package ctl-server

server := "ctl-server.nertsal.com"
server_user := "nertsal"

deploy-server:
    just build-server
    rsync -avz docker-target/release/ctl-server {{server_user}}@{{server}}:close-to-server/
    ssh {{server_user}}@{{server}} systemctl --user restart close-to-server

publish-web:
    CONNECT=wss://{{server}} cargo geng build --release --platform web --out-dir `pwd`/target/geng
    butler -- push `pwd`/target/geng nertsal/close-to-light:html5
