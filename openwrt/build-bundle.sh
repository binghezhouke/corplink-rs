#!/usr/bin/env bash
# HOST-side: build the fully-static binary, strip it, drop it into the bundle,
# and produce corplink-openwrt.tar.gz to scp to the router.
#
#   ./build-bundle.sh          # reuse existing static binary if present
#   FORCE=1 ./build-bundle.sh  # always rebuild the binary
#   NOSTRIP=1 ./build-bundle.sh # keep symbols (larger)
set -euo pipefail

cd "$(dirname "$0")"                       # openwrt/
ROOT="$(cd .. && pwd)"                     # repo root
BIN_SRC="$ROOT/target/x86_64-unknown-linux-gnu/release/corplink-rs"
BIN_DST="files/usr/bin/corplink-rs"

if [[ "${FORCE:-0}" == "1" || ! -f "$BIN_SRC" ]]; then
	echo ">> building static binary (build-static.sh)"
	( cd "$ROOT" && ./build-static.sh )
fi

mkdir -p "$(dirname "$BIN_DST")"
cp "$BIN_SRC" "$BIN_DST"
[[ "${NOSTRIP:-0}" == "1" ]] || strip "$BIN_DST" 2>/dev/null || true
chmod 755 "$BIN_DST"

echo ">> bundled binary:"
file "$BIN_DST"
if ldd "$BIN_DST" 2>&1 | grep -qv "statically linked"; then
	echo "!! WARNING: binary is NOT fully static -- it may not run on OpenWrt (musl)."
fi

TARBALL="corplink-openwrt.tar.gz"
tar czf "$TARBALL" install.sh uninstall.sh README.md files
echo
echo ">> wrote $(pwd)/$TARBALL"
echo "   scp '$TARBALL' root@ROUTER:/tmp/ && ssh root@ROUTER 'cd /tmp && tar xzf $TARBALL && sh install.sh'"
