default: build

build:
    cargo build

check:
    cargo clippy --all-targets
    ./tokei_check.sh 800 src/

test-ut room="room_ruins1":
    cargo build
    ./target/debug/gm2tiled \
        --input /home/aik/Games/UNDERTALE/data.win \
        --output /tmp/gm2tiled_ut_test \
        --rooms {{room}}

test-dr room="room_torhouse":
    cargo build
    ./target/debug/gm2tiled \
        --input /home/aik/Documents/dr/1data.win \
        --output /tmp/gm2tiled_dr_test \
        --rooms {{room}}
