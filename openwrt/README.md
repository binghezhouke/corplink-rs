# corplink-rs on OpenWrt (x86_64)

Run corplink-rs on OpenWrt as a **global VPN gateway**, configured from LuCI, with a
log viewer and automatic firewall NAT (masquerade) for LAN clients.

This is delivered as an **install script** (not an `.apk`): a fully-static binary plus
a procd service, UCI config, firewall helper and a LuCI app.

## Why no cross-compile / no musl toolchain

`build-static.sh` produces a **fully static** x86_64 binary (`ldd` → *statically linked*).
A fully static binary carries its own libc and makes syscalls directly, so it does **not**
depend on the system libc — glibc vs musl is irrelevant, and it runs on musl-based OpenWrt
as-is. DNS is handled by the pure-Rust `trust-dns` resolver (see `Cargo.toml`), avoiding the
static-glibc NSS pitfall. Your router is x86_64, same as the build host, so there is no
cross-compilation at all.

## Build (on your x86_64 Linux build host)

```sh
cd openwrt
./build-bundle.sh          # builds the static binary, strips it, makes corplink-openwrt.tar.gz
```

Prerequisites for the binary build (host): Rust toolchain + Go (for `libwg.a`), as used by
the repo's `build-static.sh`. `build-bundle.sh` reuses an existing
`target/x86_64-unknown-linux-gnu/release/corplink-rs` if present (`FORCE=1` to rebuild).

## Install (on the router)

```sh
scp corplink-openwrt.tar.gz root@ROUTER:/tmp/
ssh root@ROUTER
cd /tmp && tar xzf corplink-openwrt.tar.gz && sh install.sh
```

The installer ensures `kmod-tun` / `/dev/net/tun`, copies files, keeps any existing
`/etc/config/corplink`, refreshes LuCI/rpcd, and enables the service.

## Configure

LuCI → **Services → Corplink VPN**:

- **General**: enable, company code, username, password, platform, route mode
  (*Full tunnel* = global gateway), masquerade (NAT for LAN), log level.
- **Advanced**: interface name, explicit portal server, force udp/tcp, skip ping,
  gateway name / IP override, VPN DNS.
- **Log** tab: live `logread` output (set log level = *debug* for detail).

Set **Enable service** and **Save & Apply**. The service reloads automatically on apply.

## How it works

- `/etc/init.d/corplink` (procd) regenerates `/etc/corplink/config.json` from UCI on each
  start via `/usr/libexec/corplink-genconfig`, **preserving** the WireGuard keys, device id,
  login state and 2FA code that corplink-rs writes back into that file — so identity and
  session survive edits and reboots. Config + cookies live in `/etc/corplink` (persistent).
- `/usr/libexec/corplink-firewall` adds an fw4 zone that matches the TUN by **device name**
  (the TUN is created by corplink-rs, not netifd) with `masq` + `lan → corplink` forwarding.
  The VPN side is treated as untrusted: inbound and forwarded traffic **from** the tunnel is
  silently **DROP**ped (only LAN-initiated flows go out; their replies are allowed by fw4's
  global established/related rule). Removed automatically when the service stops or masquerade
  is turned off.
- Service stop sends SIGTERM → corplink-rs disconnects and logs out gracefully.

## Notes

- `use_vpn_dns` is **off** by default: on OpenWrt it renames `/etc/resolv.conf`, which can
  conflict with dnsmasq. Turn on only if you need VPN-pushed DNS.
- Switching accounts: run `sh uninstall.sh --purge` (removes config + keys/state) then
  reinstall, or delete `/etc/corplink/config.json` and `/etc/corplink/corplink_cookies.json`.

## Uninstall

```sh
sh uninstall.sh            # keep config + state
sh uninstall.sh --purge    # also remove /etc/config/corplink and /etc/corplink
```
