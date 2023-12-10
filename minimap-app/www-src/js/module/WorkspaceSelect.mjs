import * as Surplus from 'surplus';

import I from 'minimap/js/util/i18n.mjs';

import Logo from 'minimap/js/component/Logo.mjs';

import * as C from './WorkspaceSelect.css';

const TYPE_TAGS = {
	git: ({ remote }) => [
		<div className={C.workspaceType}>{I`Git Repository`}</div>,
		<div className={C.property}>
			<strong>{I`Remote:`}</strong> {remote}
		</div>
	],
	mem: ({ author, email }) => [
		<div className={C.workspaceType}>{I`In-Memory`}</div>,
		<div className={C.property}>
			<strong>{I`Author:`}</strong> {author}
		</div>,
		<div className={C.property}>
			<strong>{I`E-Mail:`}</strong> {email}
		</div>
	]
};

export default ({ savedWorkspaces, currentWorkspace }) => {
	return (
		<div className={C.root}>
			<div className={C.logoContainer}>
				<Logo />
			</div>
			<div className={C.listContainer}>
				<h1>{I`Select a workspace`}</h1>
				<div className={C.workspaceList}>
					{savedWorkspaces.map(workspace => {
						const TypeTag = TYPE_TAGS[workspace.type];
						return (
							<button
								className={C.workspace}
								onClick={() => currentWorkspace(workspace)}
							>
								<TypeTag {...workspace} />
							</button>
						);
					})}
				</div>
			</div>
		</div>
	);
};
