import { promises as fsp } from 'fs';

import acorn from 'acorn';
import acornJsx from 'acorn-jsx';
import * as acornWalk from 'acorn-walk';
import { extend as acornWalkJsx } from 'acorn-jsx-walk';
import compiler from 'surplus/compiler/index.js';

const JSXParser = acorn.Parser.extend(acornJsx());
acornWalkJsx(acornWalk.base);

function hasSurplusImport(code) {
	const ast = JSXParser.parse(code, {
		sourceType: 'module',
		ecmaVersion: 13
	});

	const foundMarker = {};

	try {
		acornWalk.simple(ast, {
			ImportDeclaration(n) {
				if (
					n.source.type === 'Literal' &&
					n.source.value === 'surplus'
				) {
					// Prevents needlessly spinning through the AST.
					throw foundMarker;
				}
			}
		});
	} catch (error) {
		if (error === foundMarker) {
			return true;
		} else {
			throw error;
		}
	}

	return false;
}

export default () => ({
	name: 'surplus',
	setup(build) {
		build.onLoad({ filter: /\.mjs$/ }, async args => {
			let contents = await fsp.readFile(args.path, 'utf-8');

			try {
				if (hasSurplusImport(contents)) {
					contents = compiler.compile(contents, {});
				}
			} catch (error) {
				console.error('error in file:', args.path);
				throw error;
			}

			return { contents };
		});
	}
});
