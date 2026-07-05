#[derive(serde::Deserialize, Debug)]
pub struct Resp<T> {
    pub code: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct RespCompany {
    pub name: String,
    pub zh_name: String,
    pub en_name: String,
    pub domain: String,
    pub enable_self_signed: bool,
    pub self_signed_cert: String,
    pub enable_public_key: bool,
    pub public_key: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct RespLoginMethod {
    pub login_enable_ldap: bool,
    pub login_enable: bool,
    pub login_orders: Vec<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct RespTpsLoginMethod {
    pub alias: String,
    pub login_url: String,
    pub token: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct RespCorplinkLoginMethod {
    pub mfa: bool,
    pub auth: Vec<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct RespLogin {
    #[serde(default)]
    pub url: String,
}

// response of the v1 login endpoint (/api/v1/login), e.g.
// {"result":"success","next":{"action":"GoToLink","can_skip":false}}
#[derive(serde::Deserialize, Debug)]
pub struct RespLoginV1 {
    #[serde(default)]
    pub result: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct RespOtp {
    pub url: String,
    pub code: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct RespVpnInfo {
    pub api_port: u16,
    pub vpn_port: u16,
    pub ip: String,
    // 1 for tcp, 2 for udp, we only support udp for now
    pub protocol_mode: i32,
    // localized (often Chinese) name
    pub name: String,
    // english name; many servers leave this empty
    pub en_name: String,
    pub icon: String,
    pub id: i32,
    pub timeout: i32,
}

impl RespVpnInfo {
    /// Human-readable label for logs and `vpn_server_name` matching. The server
    /// frequently ships an empty `en_name` (only the localized `name` is set),
    /// which used to make logs print an empty string (e.g. `try connect to ,`)
    /// and made `vpn_server_name` impossible to match. Fall back to `name`.
    pub fn display_name(&self) -> &str {
        if self.en_name.is_empty() {
            &self.name
        } else {
            &self.en_name
        }
    }

    /// Whether the user-configured `vpn_server_name` selects this gateway.
    /// Matches against both the english and localized names so a user can pick
    /// a server by whichever name the server actually populated.
    pub fn matches_name(&self, wanted: &str) -> bool {
        self.en_name == wanted || self.name == wanted
    }

    /// Server-advertised wireguard transport for this gateway, for logging.
    pub fn protocol_mode_str(&self) -> &'static str {
        match self.protocol_mode {
            1 => "tcp",
            2 => "udp",
            _ => "unknown",
        }
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct RespWgExtraInfo {
    pub vpn_mtu: u32,
    pub vpn_dns: String,
    pub vpn_dns_backup: String,
    pub vpn_dns_domain_split: Option<Vec<String>>,
    pub vpn_route_full: Vec<String>,
    pub vpn_route_split: Vec<String>,
    pub v6_route_full: Option<Vec<String>>,
    pub v6_route_split: Option<Vec<String>>,
}

#[derive(serde::Deserialize, Debug)]
pub struct RespWgInfo {
    pub ip: String,
    pub ipv6: String,
    pub ip_mask: String,
    pub public_key: String,
    pub setting: RespWgExtraInfo,
    pub mode: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vpn(name: &str, en_name: &str, protocol_mode: i32) -> RespVpnInfo {
        RespVpnInfo {
            api_port: 8001,
            vpn_port: 8002,
            ip: "10.0.0.1".to_string(),
            protocol_mode,
            name: name.to_string(),
            en_name: en_name.to_string(),
            icon: String::new(),
            id: 1,
            timeout: 5,
        }
    }

    #[test]
    fn display_name_falls_back_to_name_when_en_name_empty() {
        assert_eq!(vpn("成都-电信", "", 2).display_name(), "成都-电信");
        assert_eq!(vpn("成都-电信", "Chengdu", 2).display_name(), "Chengdu");
    }

    #[test]
    fn matches_name_checks_both_names() {
        let v = vpn("成都-电信", "", 2);
        assert!(v.matches_name("成都-电信"));
        assert!(!v.matches_name("Chengdu"));

        let v = vpn("成都-电信", "Chengdu", 2);
        assert!(v.matches_name("Chengdu"));
        assert!(v.matches_name("成都-电信"));
    }

    #[test]
    fn protocol_mode_str_maps_known_modes() {
        assert_eq!(vpn("a", "", 1).protocol_mode_str(), "tcp");
        assert_eq!(vpn("a", "", 2).protocol_mode_str(), "udp");
        assert_eq!(vpn("a", "", 9).protocol_mode_str(), "unknown");
    }
}
