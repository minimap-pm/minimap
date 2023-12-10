import * as Surplus from 'surplus';
import S from 's-js';
import { localSignal } from 's-storage';

import I from 'minimap/js/util/i18n.mjs';
import systemTheme from 'minimap/js/util/s-theme.mjs';

import Root from 'minimap/js/module/Root.mjs';
import ErrorView from 'minimap/js/module/ErrorView.mjs';

import { MemoryWorkspace } from 'minimap/js/api.mjs';

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
	window.doItNow = async () => {
		console.log('opening memory workspace...');
		const workspace = await MemoryWorkspace.open(
			'Max Mustermann',
			'max@example.com'
		);
		console.log('opened workspace with name:', await workspace.getName());
		console.log('setting workspace name...');
		await workspace.setName('Test Workspace');
		console.log('workspace name is now:', await workspace.getName());
		console.log("creating project 'TEST'...");
		const project = await workspace.createProject('TEST');
		console.log('creating a ticket');
		const ticket = await project.createTicket();
		console.log('ticket slug:', ticket.slug);
		console.log('setting ticket title and first comment...');
		await ticket.setTitle('Test Ticket');
		await ticket.addComment('This is a test ticket.');
		console.log('ticket title:', await ticket.getTitle());
		console.log('ticket comments:', await ticket.getComments());
	};
});
