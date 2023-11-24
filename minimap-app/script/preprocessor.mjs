/*
	Kitchen sink codebase pre-processor
	and linter.
*/

import { promises as fsp } from 'fs';

import arg from 'arg';
import acorn from 'acorn';
import acornJsx from 'acorn-jsx';
import * as acornWalk from 'acorn-walk';
import chalkModule from 'chalk';
import { extend as acornWalkJsx } from 'acorn-jsx-walk';
import globby from 'globby';
import htmlentities from 'htmlentities';

import makeId from '@minimap/web/js/util/i18n-id.mjs';
import { readTSV, writeTSV } from './tsv.mjs';

const chalk = chalkModule.stderr;

const JSXParser = acorn.Parser.extend(acornJsx());
acornWalkJsx(acornWalk.base);

const args = arg({
	'--lang-file': String,
	'--tag': String
});

const requireArg = name => {
	if (!args[name]) throw new Error(`missing required arg: ${name}`);
};

requireArg('--lang-file');
requireArg('--tag');

const files = (await Promise.all(args._.map(glob => globby(glob)))).flat();

const errors = [];

async function processFile(filename) {
	const contents = await fsp.readFile(filename, 'utf-8');

	const ast = (() => {
		try {
			return JSXParser.parse(contents, {
				sourceType: 'module',
				ecmaVersion: 13,
				locations: true
			});
		} catch (err) {
			err.filename = filename;
			throw err;
		}
	})();

	const strings = [];

	const addQuasi = quasi => {
		const chunks = quasi.quasis.map(q => {
			if (q.type !== 'TemplateElement') {
				throw new Error(
					`discovered non-TemplateElement tagged quasi element: ${q.type}:\n${q}`
				);
			}

			return q.value.raw;
		});

		strings.push(makeId(chunks));
	};

	acornWalk.simple(ast, {
		TaggedTemplateExpression(n) {
			/* detect and convert i18n strings so they can be added to the translations file */
			if (n?.tag?.type === 'Identifier' && n.tag.name === args['--tag']) {
				addQuasi(n.quasi);
			}
		},

		ImportDeclaration(n) {
			if (n.source.type !== 'Literal') {
				errors.push([
					filename,
					n,
					'import declaration sources must be literals'
				]);
				return;
			}
		}
	});

	return strings;
}

const sourceKeys = new Set((await Promise.all(files.map(processFile))).flat());

/* check and emit errors */
if (errors.length > 0) {
	for (const error of errors) {
		console.error(
			chalk`${error[0]}:${error[1].loc.start.line}:${error[1].loc.start.column}: {redBright.bold error:} {whiteBright ${error[2]}}`
		);
	}
	console.error();
	process.exit(1);
}

const contents = await fsp.readFile(args['--lang-file'], 'utf-8');
const corpus = readTSV(contents, 'key');
const corpusKeys = new Set(Object.keys(corpus));

const toAdd = new Set([...sourceKeys].filter(k => !corpusKeys.has(k)));
const toRemove = new Set([...corpusKeys].filter(k => !sourceKeys.has(k)));

for (const key of toAdd.values()) {
	corpus[key] = {};
}

for (const key of toRemove.values()) {
	delete corpus[key];
}

const tsvContents = writeTSV(corpus, 'key');

await fsp.writeFile(args['--lang-file'], tsvContents, 'utf-8');
