default: build

build:
    cargo build

check:
    cargo clippy --all-targets
    ./tokei_check.sh 800 src/

test-ut room="room_ruins1":
    cargo build
    ./target/debug/gm2tiled \
        --input test_sources/undertale/data.win \
        --output /tmp/gm2tiled_ut_test \
        --rooms {{room}}

test-dr chapter="1" room="room_torhouse":
    cargo build
    ./target/debug/gm2tiled \
        --input test_sources/deltarune/ch{{chapter}}.win \
        --output /tmp/gm2tiled_dr_test_ch{{chapter}} \
        --rooms {{room}}

validate-ut room="room_water3":
    cargo build
    ./scripts/run_sprite_validation.sh test_sources/undertale/data.win {{room}}

validate-dr chapter="1" room="ROOM_INITIALIZE":
    cargo build
    ./scripts/run_sprite_validation.sh test_sources/deltarune/ch{{chapter}}.win {{room}}

pixel-diff-ut:
    cargo build --release --bin gm2tiled --bin gm2tiled_regress
    ./target/release/gm2tiled_regress \
        --input test_sources/undertale/data.win \
        --output test_output/undertale \
        --dataset undertale \
        --gm2tiled-bin ./target/release/gm2tiled

pixel-diff-suite:
    cargo build --release --bin gm2tiled --bin gm2tiled_regress
    ./scripts/run_pixel_diff_suite.sh
