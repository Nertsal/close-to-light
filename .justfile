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


# Build the Demo version of the game for all platforms
build-demo:
    just build-all-platforms ./target/release-demo -F demo

# Build the Full version of the game for all platforms
build-game:
    just build-all-platforms ./target/release-game

# Make builds for every target platform
build-all-platforms TARGET_DIR *ARGS:
    # Steam-Linux
    LEADERBOARD_URL="https://ctl-server.nertsal.com" CARGO_TARGET_DIR={{TARGET_DIR}}/linux \
    cargo geng build --release --platform linux -- -F steam {{ARGS}}
    cd {{TARGET_DIR}}/linux/geng && zip -FS -r ../../linux.zip ./*
    # Steam-Windows
    docker run --rm -it -v `pwd`:/src --workdir /src \
    --env CARGO_TARGET_DIR={{TARGET_DIR}}/windows \
    --env LEADERBOARD_URL="https://ctl-server.nertsal.com" \
    ghcr.io/geng-engine/cargo-geng@sha256:ce60c252b0ac348e7dfbd6828d5473d3b6531d67502cf9efa9aece7cada6706c \
    cargo geng build --release --platform windows -- -F steam {{ARGS}}
    cd {{TARGET_DIR}}/windows/geng && zip -FS -r ../../windows.zip ./*
    # Itch-Web
    LEADERBOARD_URL="https://ctl-server.nertsal.com" CARGO_TARGET_DIR={{TARGET_DIR}}/web \
    just web build --release -F itch {{ARGS}}
    # zip -r {{TARGET_DIR}}/web.zip {{TARGET_DIR}}/web/*


build-windows *ARGS:
    docker run --rm -it -v `pwd`:/src --workdir /src --env CARGO_TARGET_DIR=./target/windows ghcr.io/geng-engine/cargo-geng@sha256:ce60c252b0ac348e7dfbd6828d5473d3b6531d67502cf9efa9aece7cada6706c \
    cargo geng build --release --platform windows -- {{ARGS}}

proton *args:
    STEAM_COMPAT_CLIENT_INSTALL_PATH=~/.local/share/Steam \
    STEAM_COMPAT_DATA_PATH=~/.local/share/Steam/steamapps/compatdata \
    steam-run '~/.local/share/Steam/steamapps/common/Proton - Experimental/proton' run {{args}}

run-windows:
    just build-windows
    just proton run target/windows/geng/close-to-light.exe
    

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
