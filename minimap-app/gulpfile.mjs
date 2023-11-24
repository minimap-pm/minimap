import { URL } from 'url';
import path from 'path';

import gulp from 'gulp';
import htmlnano from 'gulp-htmlnano';
import toIco from 'gulp-to-ico';
import esbuild from 'gulp-esbuild';
import replace from 'gulp-replace';

import i18nToJson from './script/i18n-to-json.mjs';
import getSupportedLanguages from './script/enumerate-i18n.mjs';
import listThemes from './script/list-themes.mjs';
import hashFileName from './script/hash-filename.mjs';

import esbuildSurplus from './script/esbuild-surplus.mjs';
import esbuildConsts from './script/esbuild-consts.mjs';
import esbuildCssModules from './script/esbuild-css-modules.mjs';

// The language codes that correspond to the default language
// of the app. For example, if the string literals used in the
// i18n tag template utility (I`some text here`) is US English,
// then specify ['en', 'en-US'] here.
const DEFAULT_LANGS = ['en', 'en-US'];

const env = (name, def) => {
	if (name in process.env) return process.env[name];
	if (def === undefined)
		throw new Error(`missing required environment variable: ${name}`);
	return def;
};

const dev = env('MM_DEVELOPMENT', 0) === '1';
const local = env('MM_LOCAL', 0) === '1';

const themeList = await listThemes('./www-src/js/theme.css');
const hashes = {};

const appConfig = {
	hostname: dev ? 'minimal.local' : 'minimap.pm',
	appBase: new URL(
		local ? 'http://app.minimal.local:8080' : 'https://app.minimap.pm'
	).origin,
	apiBase: new URL(
		local ? 'http://api.minimal.local:8080' : 'https://api.minimap.pm'
	).origin,
	defaultLanguages: DEFAULT_LANGS,
	supportedLanguages: getSupportedLanguages('./www-src/i18n.tsv', {
		defaultLangs: DEFAULT_LANGS
	}),
	themes: themeList
};

const makeJSBundle = ({ main, output, css = false, surplus = false }) =>
	gulp
		.src(main)
		.pipe(
			esbuild({
				entryPoints: [main],
				outfile: output,
				format: 'iife',
				sourcemap: dev && 'inline',
				minify: !dev,
				plugins: [
					...(surplus ? [esbuildSurplus()] : []),
					esbuildConsts({ appConfig }),
					...(css ? [esbuildCssModules()] : [])
				],
				bundle: true
			})
		)
		.pipe(hashFileName({ dict: hashes }))
		.pipe(gulp.dest('./build'));

export const buildClientJS = () =>
	makeJSBundle({
		main: './www-src/js/index.mjs',
		output: 'minimap.js',
		surplus: true,
		css: true
	});

export const buildClientHTML = () =>
	gulp
		.src('./www-src/index.html') // DO NOT SUFFIX
		.pipe(
			replace(/\{\{MM_HASH:([^}]+?)\s*\}\}/g, (_, name) => {
				const v = hashes[name];
				if (v) return v;
				throw new Error(
					`unknown MM_HASH requested in index.html: ${name}`
				);
			})
		)
		.pipe(
			htmlnano({
				collapseWhitespace: 'all',
				removeComments: 'all',
				removeRedundantAttributes: true,
				minifyCss: {
					preset: 'default'
				},
				minifySvg: {}
			})
		)
		.pipe(gulp.dest('./build'));

export const buildClientIcon = () =>
	gulp
		.src('./www-src/img/ico.png')
		.pipe(
			// DO NOT SUFFIX
			toIco('favicon.ico', {
				resize: true,
				sizes: [16, 24, 32, 64, 128, 256]
			})
		)
		.pipe(gulp.dest('./build'));

export const buildClientFonts = () =>
	// DO NOT SUFFIX
	gulp.src('./www-src/font/*.ttf').pipe(gulp.dest('./build/font'));

export const buildClientLangs = () =>
	gulp
		.src('./www-src/i18n.tsv')
		.pipe(i18nToJson())
		.pipe(gulp.dest('./build/lang'));

export default gulp.series(
	gulp.parallel(
		buildClientJS,
		buildClientIcon,
		buildClientFonts,
		buildClientLangs
	),
	buildClientHTML
);
