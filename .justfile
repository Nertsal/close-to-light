list:
    just --list

deploy-server:
    docker build . --tag ctl-server
    docker run --name ctl-server ctl-server
    docker cp ctl-server:target/release/ctl-server /tmp/server
    docker rm ctl-server
    rsync --rsh 'ssh -i ~/.ssh/id_kuviserver' /tmp/server nertboard.kuviman.com:~/close-to-server/server
    ssh -i ~/.ssh/id_kuviserver nertsal@nertboard.kuviman.com -t systemctl --user restart close-to-server
