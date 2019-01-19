set -ex

if [ "$RUSTFMT" = "yes" ]; then
    cargo fmt --all -- --check
elif [ "$CLIPPY" = "yes" ]; then
      cargo clippy --all -- -D warnings
else
    cargo test
fi