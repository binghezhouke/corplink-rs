#!/bin/sh
# ROUTER-side installer. Run from the extracted bundle directory:
#   tar xzf corplink-openwrt.tar.gz && sh install.sh
set -e
cd "$(dirname "$0")"

[ -f files/usr/bin/corplink-rs ] || {
	echo "ERROR: files/usr/bin/corplink-rs missing."
	echo "Run ./build-bundle.sh on your build host first, then copy the tarball over."
	exit 1
}

echo ">> ensuring TUN support (/dev/net/tun)"
if [ ! -c /dev/net/tun ]; then
	if command -v apk >/dev/null 2>&1; then
		apk add kmod-tun 2>/dev/null || true
	elif command -v opkg >/dev/null 2>&1; then
		opkg update >/dev/null 2>&1 && opkg install kmod-tun || true
	fi
	modprobe tun 2>/dev/null || true
fi
[ -c /dev/net/tun ] || echo "!! WARNING: /dev/net/tun still missing; install kmod-tun manually."

echo ">> installing files"
# copy everything except the default UCI config (never clobber user settings)
( cd files && find . -type f ) | while read -r f; do
	rel="${f#./}"
	[ "$rel" = "etc/config/corplink" ] && continue
	dst="/$rel"
	mkdir -p "$(dirname "$dst")"
	cp -a "files/$rel" "$dst"
done

# install default UCI config only on first install
if [ ! -f /etc/config/corplink ]; then
	cp -a files/etc/config/corplink /etc/config/corplink
	echo "   installed default /etc/config/corplink"
else
	echo "   kept existing /etc/config/corplink"
fi

chmod 755 /usr/bin/corplink-rs \
	/usr/libexec/corplink-genconfig \
	/usr/libexec/corplink-firewall \
	/etc/init.d/corplink

# normalize ownership (tarball may carry the build user's uid)
chown root:root \
	/usr/bin/corplink-rs \
	/usr/libexec/corplink-genconfig \
	/usr/libexec/corplink-firewall \
	/etc/init.d/corplink \
	/etc/config/corplink \
	/usr/share/luci/menu.d/luci-app-corplink.json \
	/usr/share/rpcd/acl.d/luci-app-corplink.json 2>/dev/null || true
chown -R root:root /www/luci-static/resources/view/corplink 2>/dev/null || true

echo ">> refreshing LuCI + rpcd"
rm -f /tmp/luci-indexcache* 2>/dev/null || true
/etc/init.d/rpcd reload 2>/dev/null || /etc/init.d/rpcd restart 2>/dev/null || true

echo ">> enabling service"
/etc/init.d/corplink enable
if [ "$(uci -q get corplink.main.enabled)" = "1" ]; then
	/etc/init.d/corplink start || true
fi

cat <<'EOF'

Done.
  1. Open LuCI -> Services -> Corplink VPN -> Settings
  2. Fill company code / username / password / platform
  3. Route mode = Full tunnel, Masquerade = on (for a LAN gateway)
  4. Enable service -> Save & Apply
  5. Check the Log tab (set Log level = debug for more detail)
EOF
