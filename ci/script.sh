set -ex

main() {
    cargo check --target $TARGET
    cargo check --target $TARGET --features rt

    if [ $TARGET = x86_64-unknown-linux-gnu ]; then
        cargo clean
        cargo build
    fi
}

main