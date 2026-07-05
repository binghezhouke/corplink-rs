'use strict';
'require view';
'require form';
'require fs';
'require uci';
'require ui';

return view.extend({
	render: function() {
		var m, s, o;

		m = new form.Map('corplink', _('Corplink VPN'),
			_('Rust Corplink client. Fill in your credentials, choose Full tunnel for a ' +
			  'global gateway, then enable the service and Save & Apply.'));

		s = m.section(form.NamedSection, 'main', 'corplink');
		s.addremove = false;
		s.tab('general', _('General'));
		s.tab('advanced', _('Advanced'));

		/* ---- General ---- */
		o = s.taboption('general', form.Flag, 'enabled', _('Enable service'));
		o.rmempty = false;

		o = s.taboption('general', form.Value, 'company_name', _('Company code'),
			_('The corporate short name / code used to locate the portal.'));
		o.rmempty = false;

		o = s.taboption('general', form.Value, 'username', _('Username'));

		o = s.taboption('general', form.Value, 'password', _('Password'),
			_('Leave empty for platforms that use QR / external auth.'));
		o.password = true;

		o = s.taboption('general', form.ListValue, 'platform', _('Platform'));
		o.value('feilian', 'feilian');
		o.value('feilian_v1', 'feilian_v1');
		o.value('ldap', 'ldap');
		o.value('lark', 'lark (feishu)');
		o.value('OIDC', 'OIDC');
		o.value('weixin', 'weixin');
		o.value('dingtalk', 'dingtalk');

		o = s.taboption('general', form.ListValue, 'route_mode', _('Route mode'));
		o.value('full', _('Full tunnel (global gateway)'));
		o.value('split', _('Split (intranet routes only)'));

		o = s.taboption('general', form.Flag, 'masquerade', _('Masquerade (NAT for LAN)'),
			_('Auto-manage a firewall zone that NATs LAN traffic out through the tunnel. ' +
			  'Required for a LAN-wide gateway.'));

		o = s.taboption('general', form.ListValue, 'log_level', _('Log level'));
		o.value('info', 'info');
		o.value('debug', _('debug (verbose)'));

		/* ---- Advanced ---- */
		o = s.taboption('advanced', form.Value, 'interface_name', _('TUN interface name'));
		o.placeholder = 'corplink';

		o = s.taboption('advanced', form.Value, 'server', _('Portal server'),
			_('Optional. Explicit portal address; leave empty to resolve from company code.'));
		o.optional = true;

		o = s.taboption('advanced', form.ListValue, 'force_protocol', _('Force WG transport'));
		o.value('', _('auto (follow server)'));
		o.value('udp', 'udp');
		o.value('tcp', 'tcp');
		o.optional = true;

		o = s.taboption('advanced', form.Flag, 'skip_ping', _('Skip gateway ping'),
			_('Use the first gateway without the pre-connect ping (for networks that ' +
			  'firewall the gateway api port).'));

		o = s.taboption('advanced', form.Value, 'vpn_server_name', _('Gateway name'),
			_('Optional. Pick a specific gateway by its name.'));
		o.optional = true;

		o = s.taboption('advanced', form.Value, 'vpn_server_ip', _('Gateway IP override'),
			_('Optional. Force the gateway endpoint IP (bypasses portal load-balancing).'));
		o.optional = true;
		o.datatype = 'ipaddr';

		o = s.taboption('advanced', form.DynamicList, 'vpn_route_override', _('VPN route override'),
			_('Install EXACTLY these routes (CIDRs) instead of the server-derived ones, ' +
			  'decoupled from the WireGuard AllowedIPs. Pair with Route mode = Full tunnel. ' +
			  'Leave empty to use the routes the server pushes.'));
		o.datatype = 'cidr';
		o.placeholder = '10.0.0.0/8';

		o = s.taboption('advanced', form.DynamicList, 'vpn_disallowed_routes', _('Disallowed routes'),
			_('CIDRs to carve OUT of the tunnel routes (never routed through the VPN).'));
		o.datatype = 'cidr';
		o.placeholder = '192.168.2.0/24';

		o = s.taboption('advanced', form.Flag, 'use_vpn_dns', _('Use VPN DNS'),
			_('Renames /etc/resolv.conf to push VPN DNS. May conflict with the OpenWrt ' +
			  'resolver (dnsmasq); leave off unless you know you need it.'));

		return m.render();
	},

	// A procd reload trigger only fires for an already-running service, so enabling
	// the service in the form and hitting Save & Apply would NOT start a stopped one.
	// Restart it explicitly after the uci changes are applied so "Enable" takes effect
	// immediately. The ACL grants exec on /etc/init.d/corplink.
	handleSaveApply: function(ev, mode) {
		return this.handleSave(ev).then(function() {
			return uci.apply();
		}).then(function() {
			return fs.exec('/etc/init.d/corplink', [ 'restart' ]);
		}).then(function() {
			ui.addNotification(null, E('p', {}, _('Corplink service restarted.')), 'info');
		}).catch(function(e) {
			ui.addNotification(null, E('p', {}, _('Apply failed: ') + (e.message || e)));
		});
	}
});
