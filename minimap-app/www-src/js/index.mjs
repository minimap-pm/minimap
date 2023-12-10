import * as Surplus from 'surplus';
import S from 's-js';

import I from 'minimap/js/util/i18n.mjs';
import systemTheme from 'minimap/js/util/s-theme.mjs';
import { signal as configSignal } from 'minimap/js/util/tauri-config-signal.mjs';

import Root from 'minimap/js/module/Root.mjs';
import ErrorView from 'minimap/js/module/ErrorView.mjs';

import './reset.css';
import './global.css';
import './theme.css';

const XFormString = {
	stringify: v => v ?? '',
	parse: v => v ?? ''
};

S.root(() => {
	const Minimap = {
		errorMessage: S.value(),
		theme: configSignal('theme', 'system'),
		langOverride: configSignal('lang')
	};

	if (typeof window !== 'undefined') window.Minimap = Minimap;

	// React to theme changes
	S(() => {
		document.body.classList.value = `mm theme ${
			Minimap.theme() === 'system' ? systemTheme() : Minimap.theme()
		}`;
	});

	// React to language overrides
	S(() => {
		if (Minimap.langOverride()) {
			I.language(Minimap.langOverride());
		}
	});

	// Calculate the current main view
	const currentView = S(() => {
		if (Minimap.errorMessage()) return <ErrorView {...{ Minimap }} />;

		// TODO Add your default view here.
		return () => <div>Hello, world!</div>;
	});

	// Attach!
	document.body.prepend(<Root view={currentView} fadeTime={150} />);
});
