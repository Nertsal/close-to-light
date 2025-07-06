list:
    just --list

game *ARGS:
    cargo run -- {{ARGS}}

web command *ARGS:
    cargo geng {{command}} --platform web --release -- {{ARGS}}

server PORT *ARGS:
    cargo run --release --package ctl-server {{PORT}} -- {{ARGS}}

build-server:
    docker build . --tag ctl-server
    docker run --name ctl-server -it \
        -v $(pwd):/src:ro \
        -v ctl-server-target:/target \
        -e CARGO_TARGET_DIR=/target \
        -w /src \
        ctl-server cargo build --release --package ctl-server
    docker cp ctl-server:target/release/ctl-server /tmp/server
    docker rm ctl-server

deploy-server:
    just build-server
    rsync --rsh 'ssh -i ~/.ssh/id_kuviserver' /tmp/server nertboard.kuviman.com:~/close-to-server/server
    ssh -i ~/.ssh/id_kuviserver nertsal@nertboard.kuviman.com -t systemctl --user restart close-to-server
