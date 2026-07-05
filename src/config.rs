use std::fmt;
use tokio::fs;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::state::State;
use crate::utils;

const DEFAULT_DEVICE_NAME: &str = "DollarOS";
const DEFAULT_INTERFACE_NAME: &str = "corplink";

pub const PLATFORM_LDAP: &str = "ldap";
pub const PLATFORM_CORPLINK: &str = "feilian";
// new feilian login that uses the v1 API (/api/v1/login with an AES-encrypted
// password), as served by the newer feilian backend. opt-in via config.
pub const PLATFORM_CORPLINK_V1: &str = "feilian_v1";
pub const PLATFORM_OIDC: &str = "OIDC";
// aka feishu
pub const PLATFORM_LARK: &str = "lark";
#[allow(dead_code)]
pub const PLATFORM_WEIXIN: &str = "weixin";
// aka dingding
#[allow(dead_code)]
pub const PLATFORM_DING_TALK: &str = "dingtalk";
// unknown
#[allow(dead_code)]
pub const PLATFORM_AAD: &str = "aad";

pub const STRATEGY_LATENCY: &str = "latency";
pub const STRATEGY_DEFAULT: &str = "default";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum RouteMode {
    /// Only intranet routes returned by the server (mimics official split mode).
    #[default]
    Split,
    /// Full-tunnel routes from the server (typically 0.0.0.0/0, ::/0).
    Full,
}

impl fmt::Display for RouteMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RouteMode::Split => write!(f, "split"),
            RouteMode::Full => write!(f, "full"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub company_name: String,
    pub username: String,
    pub password: Option<String>,
    pub platform: Option<String>,
    pub code: Option<String>,
    pub device_name: Option<String>,
    pub device_id: Option<String>,
    pub public_key: Option<String>,
    pub private_key: Option<String>,
    pub server: Option<String>,
    pub interface_name: Option<String>,
    pub debug_wg: Option<bool>,
    #[serde(skip_serializing)]
    pub conf_file: Option<String>,
    pub state: Option<State>,
    pub vpn_server_name: Option<String>,
    pub vpn_select_strategy: Option<String>,
    pub use_vpn_dns: Option<bool>,
    pub dns_backup_filename: Option<String>,
    pub auto_setup_routes: Option<bool>,
    /// "split" (default) or "full". Selects which route list from the server to apply.
    pub route_mode: Option<RouteMode>,
    /// Optional list of CIDR routes to exclude from AllowedIPs / system routes.
    /// Useful in full mode to punch holes for local LAN or the VPN peer IP itself,
    /// avoiding routing loops (e.g. 192.168.1.0/24, 10.0.0.5/32).
    pub vpn_disallowed_routes: Option<Vec<String>>,
    /// When set, run entirely in userspace (gVisor netstack) and expose a SOCKS5
    /// proxy at this listen address (e.g. "0.0.0.0:1080" or "127.0.0.1:1080")
    /// instead of creating a kernel TUN device. No system interface, routes, DNS
    /// changes or root privileges are required. Only TCP CONNECT is supported.
    pub socks5_listen: Option<String>,
    /// Optional SOCKS5 username/password authentication (RFC 1929). When
    /// `socks5_username` is set and non-empty, clients must authenticate with
    /// these credentials; otherwise the proxy accepts connections without auth.
    pub socks5_username: Option<String>,
    pub socks5_password: Option<String>,
    /// Force the WireGuard transport protocol instead of using the server-advertised
    /// `protocol_mode`. Accepts "udp" or "tcp" (case-insensitive). Some `protocol_mode: 1`
    /// (TCP) gateways also accept WireGuard over UDP -- for those the server even ships a
    /// `protocol_detect_config` (udp<->tcp switch thresholds) in the `/api/vpn/list` entry.
    /// Since WireGuard-over-TCP can collapse to a few KB/s on a lossy uplink (TCP-over-TCP
    /// head-of-line blocking), forcing "udp" can be far faster there. Leave unset to keep the
    /// default (follow server `protocol_mode`: 1 => tcp, otherwise udp).
    pub force_protocol: Option<String>,
    /// Skip the pre-connect gateway ping. Gateway selection normally requires a
    /// successful HTTPS ping to the gateway's `api_port`; some networks firewall
    /// that api port even though the WireGuard transport port itself is reachable,
    /// which makes selection fail with "no vpn available" despite a usable tunnel.
    /// When true, the ping is skipped and the first gateway (after `vpn_server_name`
    /// filtering) is used directly. Best combined with an explicit `vpn_server_name`.
    pub skip_ping: Option<bool>,
    /// Override the gateway endpoint IP returned by `/api/vpn/list`. The portal load-
    /// balances a gateway across several public IPs and picks one based on the client's
    /// apparent source; that IP can be firewalled from the current host even though a
    /// different IP of the *same* gateway is directly reachable. When set, this IP is used
    /// for both the ConnectVPN/keepalive API calls (on `api_port`) and the WireGuard peer
    /// endpoint (on `vpn_port`), while the ports come from the selected `/api/vpn/list`
    /// entry. Pair with `vpn_server_name` (to pick the gateway) and, on UDP-blocked
    /// networks, `force_protocol: "tcp"`. Leave unset to use the server-provided IP.
    pub vpn_server_ip: Option<String>,
    /// Replace the system routes installed for the tunnel with this custom list,
    /// decoupling them from the WireGuard AllowedIPs. Only affects the OS routing
    /// table (and netstack accept-list), NOT the AllowedIPs sent to wg. Intended
    /// pairing: `route_mode: "full"` so AllowedIPs becomes 0.0.0.0/0 (wg accepts all
    /// traffic and drops nothing), then this list narrows what the OS actually sends
    /// into the interface. The peer endpoint and `vpn_disallowed_routes` are still
    /// carved out of this list to avoid routing loops. Ignored when empty or when
    /// `auto_setup_routes` is false (which installs no routes at all).
    pub vpn_route_override: Option<Vec<String>>,
    /// Default log verbosity when the `RUST_LOG` env var is not set. Accepts any
    /// env_logger filter string, e.g. "info" (default), "debug", "warn", or a
    /// per-module directive like "corplink_rs=debug". `RUST_LOG`, when present,
    /// always takes precedence over this value. Bump to "debug" to surface the
    /// verbose diagnostics (cookies, tokens, keep-alive) that are hidden by default.
    pub log_level: Option<String>,
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match serde_json::to_string_pretty(self) {
            Ok(s) => write!(f, "{}", s),
            Err(e) => write!(f, "<invalid config: {e}>")
        }
    }
}

impl Config {
    pub async fn from_file(file: &str) -> Result<Config> {
        let conf_str = fs::read_to_string(file)
            .await
            .with_context(|| format!("failed to read config file {file}"))?;

        let mut conf: Config = serde_json::from_str(&conf_str[..])
            .with_context(|| format!("failed to parse config file {file}"))?;

        conf.conf_file = Some(file.to_string());
        let mut update_conf = false;
        if conf.interface_name.is_none() {
            conf.interface_name = Some(DEFAULT_INTERFACE_NAME.to_string());
            update_conf = true;
        }
        if conf.device_name.is_none() {
            conf.device_name = Some(DEFAULT_DEVICE_NAME.to_string());
            update_conf = true;
        }
        if conf.device_id.is_none() {
            let device_name = conf
                .device_name
                .as_ref()
                .context("device name missing when generating device id")?;
            conf.device_id = Some(format!("{:x}", md5::compute(device_name)));
            update_conf = true;
        }
        match &conf.private_key {
            Some(private_key) => match conf.public_key {
                Some(_) => {
                    // both keys exist, do nothing
                }
                None => {
                    // only private key exists, generate public from private
                    let public_key = utils::gen_public_key_from_private(private_key)?;
                    conf.public_key = Some(public_key);
                    update_conf = true;
                }
            },
            None => {
                // no key exists, generate new
                let (public_key, private_key) = utils::gen_wg_keypair();
                (conf.public_key, conf.private_key) = (Some(public_key), Some(private_key));
                update_conf = true;
            }
        }
        if update_conf {
            conf.save().await?;
        }
        Ok(conf)
    }

    pub async fn save(&self) -> Result<()> {
        let file = self
            .conf_file
            .as_ref()
            .context("config file path missing")?;
        let data = format!("{}", &self);
        fs::write(file, data)
            .await
            .with_context(|| format!("failed to write config file {file}"))?;
        Ok(())
    }
}

#[derive(Serialize, Clone)]
pub struct WgConf {
    // standard wg conf
    pub address: String,
    pub address6: String,
    pub peer_address: String,
    pub mtu: u32,
    pub public_key: String,
    pub private_key: String,
    pub peer_key: String,
    pub allowed_ips: Vec<String>,
    pub routes: Vec<String>,

    // extra confs
    pub dns: String,

    // corplink confs
    pub protocol: i32,
}
