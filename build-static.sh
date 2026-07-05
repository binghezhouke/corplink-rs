#!/usr/bin/env bash
# Build a fully-static, self-contained corplink-rs binary.
#
#   ./build-static.sh
#
# Output: target/x86_64-unknown-linux-gnu/release/corplink-rs
#         `ldd` -> "statically linked"; runs on any x86_64 Linux, no deps.
#
# Static linking is applied here via RUSTFLAGS (not .cargo/config.toml) on
# purpose: a global crt-static in config.toml also gets applied to proc-macro
# crates (e.g. async-trait), which cannot be built as static and breaks a plain
# `cargo build`. Keeping it in this script leaves normal development builds
# untouched.
#
# Why glibc-static and NOT musl-static:
#   libwg.a is a CGO archive that embeds the Go runtime. Linked into a fully
#   static *musl* binary it segfaults at startup inside the Go runtime
#   (runtime.sysargs, reading the aux vector) -- a known incompatibility between
#   Go c-archives and static musl (both PIE and non-PIE). Static glibc links the
#   same archive fine.
#
# DNS portability:
#   A static glibc binary would normally dlopen() libnss_*.so for getaddrinfo(),
#   which breaks across glibc versions. reqwest is built with the "trust-dns"
#   feature (Cargo.toml) so name resolution is pure-Rust and needs no NSS.
set -euo pipefail

cd "$(dirname "$0")"

TARGET=x86_64-unknown-linux-gnu

# libwg.a must be a glibc (not musl) CGO archive for the static glibc link.
# Build it if missing; pass FORCE_LIBWG=1 to rebuild.
if [[ ! -f libwg/libwg.a || "${FORCE_LIBWG:-0}" == "1" ]]; then
    echo ">> building libwg (glibc CGO archive)"
    ( cd libwg && CGO_ENABLED=1 CC="${CC:-cc}" ./build.sh )
fi

echo ">> building static corplink-rs for $TARGET"
RUSTFLAGS="-C target-feature=+crt-static" \
    cargo build --release --target "$TARGET" "$@"

BIN="target/$TARGET/release/corplink-rs"
echo ">> done: $BIN"
file "$BIN"
ldd "$BIN" 2>&1 || true
