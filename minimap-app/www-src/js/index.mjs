import * as Surplus from 'surplus';
import S from 's-js';
import { localSignal } from 's-storage';

import I from 'minimap/js/util/i18n.mjs';
import systemTheme from 'minimap/js/util/s-theme.mjs';

import Root from 'minimap/js/module/Root.mjs';
import ErrorView from 'minimap/js/module/ErrorView.mjs';

import { invoke } from '@tauri-apps/api';

import './reset.css';
import './global.css';
import './theme.css';

const XFormString = {
	stringify: v => v ?? '',
	parse: v => v ?? ''
};

export const initMinimap = () => S.root(() => {
	const Minimap = {
		errorMessage: S.value(),
		theme: localSignal('minimap.theme', {
			init: 'system',
			transform: XFormString
		}),
		langOverride: localSignal('minimap.lang', {
			transform: XFormString
		})
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

	// TODO DEBUG: test tauri API
	invoke('hello').then(r => console.log('hello():', r));
	// TODO DEBUG: open a workspace
	const uri = 'git@github.com:Qix-/test-minimap.git';
	invoke('workspace_open', { uri })
		.then(r => {
			console.log('workspace_open():', r);
			invoke('project_create', { uri, slug: 'test-project' })
				.then(r => console.log('project_create():', r))
				.catch(err => console.error(err));
		})
		.catch(err => console.error(err));
});

setTimeout(initMinimap, 1000);
