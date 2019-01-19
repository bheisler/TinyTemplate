set -ex

if [ "$RUSTFMT" = "yes" ]; then
    rustup component add rustfmt-preview
fi

if [ "$CLIPPY" = "yes" ]; then
    rustup component add clippy-preview
fi