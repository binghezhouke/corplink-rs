#!/bin/sh
# ROUTER-side uninstaller.
#   sh uninstall.sh            # remove program + LuCI, keep config/state
#   sh uninstall.sh --purge    # also remove /etc/config/corplink and /etc/corplink
/etc/init.d/corplink stop 2>/dev/null || true
/usr/libexec/corplink-firewall remove 2>/dev/null || true
/etc/init.d/corplink disable 2>/dev/null || true

rm -f /usr/bin/corplink-rs \
	/usr/libexec/corplink-genconfig \
	/usr/libexec/corplink-firewall \
	/etc/init.d/corplink \
	/usr/share/luci/menu.d/luci-app-corplink.json \
	/usr/share/rpcd/acl.d/luci-app-corplink.json
rm -rf /www/luci-static/resources/view/corplink

if [ "$1" = "--purge" ]; then
	rm -rf /etc/corplink
	rm -f /etc/config/corplink
	echo "purged /etc/config/corplink and /etc/corplink (keys/state/cookies)"
else
	echo "kept /etc/config/corplink and /etc/corplink (use --purge to remove)"
fi

rm -f /tmp/luci-indexcache* 2>/dev/null || true
/etc/init.d/rpcd reload 2>/dev/null || true
echo "uninstalled."
