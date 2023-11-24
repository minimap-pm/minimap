import path from 'path';

import gulp from 'gulp';
import through2 from 'through2';
import File from 'vinyl';

import backslash from 'minimap/js/util/backslash.mjs';

import { readTSV } from './tsv.mjs';

const deconstructString = str =>
	str
		.split(/\{(\d+)\}/g)
		.map((chunk, i) =>
			i % 2 === 0 ? backslash(chunk) : parseInt(chunk, 10)
		);

export default () => {
	return through2.obj(function (file, enc, next) {
		const corpus = readTSV(file.contents.toString('utf8'), 'key');
		const base = path.join(file.path, '..');

		const langs = new Map();

		for (const key of Object.keys(corpus)) {
			const record = corpus[key];

			for (const lang of Object.keys(record)) {
				const value = record[lang];

				let langMap;

				if (langs.has(lang)) {
					langMap = langs.get(lang);
				} else {
					langMap = {};
					langs.set(lang, langMap);
				}

				const langString = record[lang];
				if (langString) {
					langMap[key] = deconstructString(langString);
				}
			}
		}

		for (const [lang, corpus] of langs.entries()) {
			this.push(
				new File({
					base,
					path: path.join(base, `${lang}.json`),
					contents: Buffer.from(JSON.stringify(corpus))
				})
			);
		}

		next();
	});
};
