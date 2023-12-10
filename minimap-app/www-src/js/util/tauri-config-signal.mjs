import S from 's-js';
import Sarray from 's-array';
import { invoke } from '@tauri-apps/api/tauri';

const currentConfig = S.value();

export async function refresh() {
	currentConfig(await invoke('config_load'));
}

export async function save() {
	await invoke('config_store', { config: S.sample(currentConfig) });
}

function setup(name, init, fn) {
	const v = fn(S.sample(currentConfig)[name] ?? init);
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

export function value(name, init) {
	return setup(name, init, S.value);
}

export function data(name, init) {
	return setup(name, init, S.data);
}

export function array(name, init) {
	return setup(name, init, Sarray);
}

await refresh();
