import * as Surplus from 'surplus';
import S from 's-js';

import I from 'minimap/js/util/i18n.mjs';
import systemTheme from 'minimap/js/util/s-theme.mjs';
import * as configSignal from 'minimap/js/util/tauri-config-signal.mjs';

import Root from 'minimap/js/module/Root.mjs';
import ErrorView from 'minimap/js/module/ErrorView.mjs';
import WorkspaceSelect from 'minimap/js/module/WorkspaceSelect.mjs';
import Loading from 'minimap/js/module/Loading.mjs';

import { Workspace } from 'minimap/js/api.mjs';

import './reset.css';
import './global.css';
import './theme.css';

S.root(() => {
	const Minimap = {
		errorMessage: S.value(),
		theme: configSignal.value('theme', 'system'),
		langOverride: configSignal.value('lang'),
		savedWorkspaces: configSignal.array('workspaces', [
			{
				type: 'git',
				remote: 'git@github.com:Qix-/test-minimap.git'
			},
			{ type: 'mem', author: 'Max Mustermann', email: 'max@example.com' },
			{
				type: 'git',
				remote: 'file:///Z:/tmp/minimap-test-repo'
			}
		]),
		currentWorkspace: S.data()
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
		if (Minimap.errorMessage()) return <ErrorView {...Minimap} />;

		const currentWorkspace = Minimap.currentWorkspace();
		if (currentWorkspace && currentWorkspace.workspace)
			return () => (
				<div>
					workspace view:{' '}
					{JSON.stringify({
						...currentWorkspace,
						workspace: undefined
					})}
				</div>
			);
		if (currentWorkspace && !currentWorkspace.workspace)
			return () => <Loading>{I`loading workspace...`}</Loading>;

		return () => <WorkspaceSelect {...Minimap} />;
	});

	// React to workspace loads
	S(async () => {
		const currentWorkspace = Minimap.currentWorkspace();
		if (!currentWorkspace) return;
		if (currentWorkspace.workspace) return;

		switch (currentWorkspace.type) {
			case 'git':
				if (!currentWorkspace.remote) {
					console.error(
						"missing required property 'remote' for workspace type 'git':",
						currentWorkspace
					);
					Minimap.currentWorkspace(undefined);
					break;
				}

				currentWorkspace.workspace = await Workspace.open_git(
					currentWorkspace.remote
				);

				Minimap.currentWorkspace(currentWorkspace);

				break;
			case 'mem':
				currentWorkspace.workspace = await Workspace.open_mem(
					currentWorkspace.author ?? '<unknown author>',
					currentWorkspace.email ?? '<unknown email>'
				);

				Minimap.currentWorkspace(currentWorkspace);

				break;
			default:
				console.error('unknown workspace type:', currentWorkspace.type);
				Minimap.currentWorkspace(undefined);
				break;
		}
	});

	// Attach!
	document.body.prepend(<Root view={currentView} fadeTime={150} />);
});
