'use strict';
'require view';
'require fs';
'require poll';
'require ui';

return view.extend({
	// no config form here
	handleSaveApply: null,
	handleSave: null,
	handleReset: null,

	render: function() {
		var log = E('pre', {
			'style': 'white-space: pre-wrap; word-break: break-word; ' +
			         'max-height: 60vh; overflow: auto; padding: .5em; ' +
			         'border: 1px solid #ccc; border-radius: 3px;'
		}, [ _('Collecting data...') ]);

		function refresh() {
			return fs.exec('/sbin/logread', [ '-e', 'corplink' ]).then(function(res) {
				var out = (res && res.stdout) ? res.stdout.trim() : '';
				log.textContent = out.length ? out : _('No corplink log entries yet.');
				log.scrollTop = log.scrollHeight;
			}).catch(function(err) {
				log.textContent = _('Failed to read log: ') + err;
			});
		}

		poll.add(refresh, 5);

		return refresh().then(function() {
			return E('div', { 'class': 'cbi-map' }, [
				E('h2', {}, _('Corplink Log')),
				E('div', { 'class': 'cbi-map-descr' },
					_('Live output from the corplink-rs service (refreshes every 5s). ' +
					  'Set Log level to "debug" in Settings for more detail.')),
				E('div', { 'class': 'cbi-section' }, [
					E('div', { 'style': 'margin-bottom:.5em' }, [
						E('button', {
							'class': 'btn cbi-button cbi-button-action',
							'click': ui.createHandlerFn(this, refresh)
						}, _('Refresh'))
					]),
					log
				])
			]);
		}.bind(this));
	}
});
