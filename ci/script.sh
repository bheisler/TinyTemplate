set -ex

export CARGO_INCREMENTAL=0

if [ "$RUSTFMT" = "yes" ]; then
    cargo fmt --all -- --check
elif [ "$CLIPPY" = "yes" ]; then
      cargo clippy --all -- -D warnings
else
    cargo test
fi