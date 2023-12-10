import S from 's-js';
import { invoke } from '@tauri-apps/api/tauri';

const currentConfig = S.value();

export async function refresh() {
	currentConfig(await invoke('config_load'));
}

export async function save() {
	await invoke('config_store', { config: S.sample(currentConfig) });
}

export function signal(name, init) {
	const v = S.value(S.sample(currentConfig)[name] ?? init);
	S.on(
		currentConfig,
		() => {
			v(currentConfig()[name] ?? init);
		},
		init,
		true
	);
	S(() => {
		const o = S.sample(currentConfig);
		o[name] = v();
		currentConfig(o);
		save();
	});
	return v;
}

await refresh();
