#!/usr/bin/env bash

set -euxo pipefail

VERSION=${REF#"refs/tags/"}
DIST=$(pwd)/dist

PROJECT_NAME="RS HDF5 plotter"
BIN="rsplotter"

echo "Packaging ${PROJECT_NAME} binary ${BIN} ${VERSION} for ${TARGET}..."

test -f Cargo.lock || cargo generate-lockfile

echo "Installing rust toolchain for ${TARGET}..."
rustup target add "${TARGET}"

if [[ $TARGET == aarch64-unknown-linux-musl ]]; then
    export CC=aarch64-linux-gnu-gcc
fi

if [[ $TARGET == armv7-unknown-linux-musleabihf ]] || [[ $TARGET == arm-unknown-linux-musleabihf ]]; then
    export CC=arm-linux-gnueabihf-gcc
fi

echo "Building ${BIN}..."
RUSTFLAGS="--deny warnings --codegen target-feature=+crt-static ${TARGET_RUSTFLAGS}" \
    cargo build --bin ${BIN} --target "${TARGET}" --release
EXECUTABLE=target/${TARGET}/release/${BIN}

if [[ $OS == windows-latest ]]; then
    EXECUTABLE=$EXECUTABLE.exe
fi
#exit 0
echo "Copying release files..."
mkdir dist
cp -r \
    "$EXECUTABLE" \
    Cargo.lock \
    Cargo.toml \
    LICENSE \
    README.md \
    "$DIST"

cd "$DIST"
echo "Creating release archive..."
case $OS in
    ubuntu-latest | macos-latest)
        ARCHIVE=$DIST/${BIN}-$VERSION-${TARGET}.tar.gz
        tar czf "$ARCHIVE" ./*
        echo "archive=$ARCHIVE" >> "$GITHUB_OUTPUT"
    ;;
    windows-latest)
        ARCHIVE=$DIST/${BIN}-$VERSION-${TARGET}.zip
        7z a "$ARCHIVE" ./*
        echo "archive=$(pwd -W)/${BIN}-$VERSION-${TARGET}.zip" >> "$GITHUB_OUTPUT"
    ;;
esac
