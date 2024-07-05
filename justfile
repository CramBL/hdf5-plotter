set dotenv-required

@_default:
    just --list --no-aliases

build-static-x86-musl *ARGS:
    RUSTFLAGS="--codegen target-feature=+crt-static" \
    cargo build --bin rsplotter --target "x86_64-unknown-linux-musl" {{ARGS}}
