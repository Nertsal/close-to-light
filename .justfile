list:
    just --list

# Test that all targets build correctly and tests pass
test:
    cargo build --release # Native build
    cargo geng build --platform web --release # Web build
    cargo test --workspace --all-features # Tests
    cargo check --workspace -F steam
    cargo check --workspace -F steam -F demo
    cargo check --workspace -F itch -F demo

game *ARGS:
    cargo run -- {{ARGS}}

web command *ARGS:
    cargo geng {{command}} --platform web --release {{ARGS}}


# Build the Demo version of the game for all platforms
build-demo:
    just build-all-platforms ./target/release-demo --features demo

# Build the Full version of the game for all platforms
build-game:
    just build-all-platforms ./target/release-game

docker_image := "ctl-build-docker"
steam_sdk := "./dev-assets/redistributable_bin"
server_url := "https://ctl-server.nertsal.com"
server := "ctl-server.nertsal.com"
server_user := "nertsal"

build-docker:
    docker build -t {{docker_image}} .

# Make builds for every target platform
build-all-platforms TARGET_DIR *ARGS:
    # Steam-Linux
    # LEADERBOARD_URL={{server_url}} CARGO_TARGET_DIR={{TARGET_DIR}}/linux \
    # cargo geng build --release --platform linux --features steam {{ARGS}}
    # cp {{steam_sdk}}/linux64/libsteam_api.so {{TARGET_DIR}}/linux/geng
    # cd {{TARGET_DIR}}/linux/geng && zip -FS -r ../../linux.zip ./*
    docker run --user $(id -u):$(id -g) --rm -it -v `pwd`:/src --workdir /src \
    --env CARGO_HOME=./target/linux/.cargo \
    --env CARGO_TARGET_DIR={{TARGET_DIR}}/linux \
    --env LEADERBOARD_URL={{server_url}} \
    {{docker_image}} \
    cargo geng build --release --platform linux --features steam {{ARGS}}
    cp {{steam_sdk}}/linux64/libsteam_api.so {{TARGET_DIR}}/linux/geng
    cd {{TARGET_DIR}}/linux/geng && zip -FS -r ../../linux.zip ./*
    # Steam-Windows
    docker run --user $(id -u):$(id -g) --rm -it -v `pwd`:/src --workdir /src \
    --env CARGO_HOME=./target/windows/.cargo \
    --env CARGO_TARGET_DIR={{TARGET_DIR}}/windows \
    --env LEADERBOARD_URL={{server_url}} \
    {{docker_image}} \
    cargo geng build --release --platform windows --features steam {{ARGS}}
    cp {{steam_sdk}}/win64/steam_api64.dll {{TARGET_DIR}}/windows/geng
    cd {{TARGET_DIR}}/windows/geng && zip -FS -r ../../windows.zip ./*
    # Itch-Web
    LEADERBOARD_URL=wss://{{server}} CARGO_TARGET_DIR={{TARGET_DIR}}/web \
    cargo geng build --release --platform web --features itch {{ARGS}}
    # zip -r {{TARGET_DIR}}/web.zip {{TARGET_DIR}}/web/*


build-windows *ARGS:
    docker run --rm -it -v `pwd`:/src --workdir /src \
    --env CARGO_HOME=./target/windows/.cargo \
    --env CARGO_TARGET_DIR=./target/windows {{docker_image}} \
    cargo geng build --release --platform windows {{ARGS}}

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

deploy-server:
    just build-server
    rsync -avz docker-target/release/ctl-server {{server_user}}@{{server}}:close-to-server/
    ssh {{server_user}}@{{server}} systemctl --user restart close-to-server

backup-server:
    rsync -r --delete {{server_user}}@{{server}}:close-to-server/server-data/ server-backup

publish-itch:
    CONNECT=wss://{{server}} cargo geng build --release --platform web --out-dir `pwd`/target/release-demo/web --features itch --features demo
    butler -- push `pwd`/target/release-demo/web nertsal/close-to-light:html5
