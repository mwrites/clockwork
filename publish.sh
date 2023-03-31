CI_TAG=v2.0.2 \
    CRATES_IO_TOKEN="cioMuUF4LdKFMUj8d7qmgSa1aOaSVvNSf7f" \
    cargo run --manifest-path=scripts/ci/publish-helper/Cargo.toml \
    --bin cargo-publish-workspace publish-workspace \
    --crate-prefix mat-clockwork- \
    --exclude mat-clockwork-thread-program-v1 \
    $@
