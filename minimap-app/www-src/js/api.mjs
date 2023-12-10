import { invoke } from '@tauri-apps/api/tauri';

class Ticket {
	constructor(workspaceId, ticketId, prefix) {
		this._workspaceId = workspaceId;
		this._ticketId = ticketId;
		this._prefix = prefix;
	}

	get slug() {
		return this._ticketId;
	}

	/*async*/ _send(command, data = {}) {
		return invoke(`${this._prefix}_${command}`, {
			workspace: this._workspaceId,
			ticket: this._ticketId,
			...data
		});
	}

	/*async*/ getTitle() {
		return this._send('ticket_title');
	}

	/*async*/ setTitle(title) {
		return this._send('ticket_set_title', { title });
	}

	/*async*/ addComment(comment) {
		return this._send('ticket_add_comment', { comment });
	}

	/*async*/ getComments() {
		return this._send('ticket_comments');
	}

	/*async*/ getStatus() {
		return this._send('ticket_status');
	}

	/*async*/ setStatus(status) {
		return this._send('ticket_set_status', { status });
	}

	/*async*/ upsertAttachment(name, data) {
		if (data instanceof ArrayBuffer) {
			data = new Uint8Array(data);
		}

		if (data instanceof Uint8Array) {
			return this._send('ticket_upsert_attachment', { name, data });
		}

		if (data instanceof String || typeof data === 'string') {
			return this._send('ticket_upsert_attachment_filepath', {
				name,
				filepath: data
			});
		}
	}

	/*async*/ removeAttachment(name) {
		return this._send('ticket_remove_attachment', { name });
	}

	/*async*/ getAttachment(name) {
		return this._send('ticket_attachment', { name });
	}

	/*async*/ getAttachmentBase64(name) {
		return this._send('ticket_attachment_base64', { name });
	}

	/*async*/ getDependencies() {
		return this._send('ticket_dependencies');
	}

	/*async*/ addDependency(origin, endpoint) {
		return this._send('ticket_add_dependency', { origin, endpoint });
	}

	/*async*/ removeDependency(origin, endpoint) {
		return this._send('ticket_remove_dependency', { origin, endpoint });
	}
}

class Project {
	constructor(workspaceId, projectId, prefix) {
		this._workspaceId = workspaceId;
		this._projectId = projectId;
		this._prefix = prefix;
	}

	/*async*/ _send(command, data = {}) {
		return invoke(`${this._prefix}_${command}`, {
			workspace: this._workspaceId,
			project: this._projectId,
			...data
		});
	}

	/*async*/ getName() {
		return this._send('project_name');
	}

	/*async*/ setName(name) {
		return this._send('project_set_name', { name });
	}

	/*async*/ getDescription() {
		return this._send('project_description');
	}

	/*async*/ setDescription(description) {
		return this._send('project_set_description', { description });
	}

	async createTicket() {
		const slug = await this._send('project_create_ticket');
		return new Ticket(this._workspaceId, slug, this._prefix);
	}
}

export class Workspace {
	static async open_git(remote) {
		const id = await invoke('git_workspace_open', { remote });
		return new GitWorkspace(id, 'git');
	}

	static async open_temporary(author, email) {
		const id = await invoke('mem_workspace_open', { author, email });
		return new Workspace(id, 'mem');
	}

	constructor(id, prefix) {
		this._id = id;
		this._prefix = prefix;
	}

	/*async*/ _send(command, data = {}) {
		return invoke(`${this._prefix}_${command}`, {
			workspace: this._id,
			...data
		});
	}

	/*async*/ getName() {
		return this._send('workspace_name');
	}

	/*async*/ setName(name) {
		return this._send('workspace_set_name', { name });
	}

	/*async*/ getDescription() {
		return this._send('workspace_description');
	}

	/*async*/ setDescription(description) {
		return this._send('workspace_set_description', { description });
	}

	async createProject(project) {
		await this._send('workspace_create_project', { project });
		return new Project(this._id, project, this._prefix);
	}

	getProject(project) {
		return new Project(this._id, project, this._prefix);
	}

	/*async*/ getProjects() {
		return this._send('workspace_projects');
	}

	/*async*/ deleteProject(project) {
		return this._send('workspace_delete_project', { project });
	}

	getTicket(ticket) {
		return new Workspace(this._id, ticket, this._prefix);
	}
}
