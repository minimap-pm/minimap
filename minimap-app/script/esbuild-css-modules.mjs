import { promises as fsp } from 'fs';
import path from 'path';

import css from 'css';
import cssWhat from 'css-what';
import { camelCase, camelCaseTransformMerge } from 'camel-case';

const toArrayBuffer = text => new TextEncoder('utf-8').encode(text);

const camelize = str => camelCase(str, { transform: camelCaseTransformMerge });

async function processCSS(contents, uid, mappings) {
	const ast = css.parse(contents);

	let priority = 0;

	ast.stylesheet.rules = ast.stylesheet.rules
		.map(rule => {
			switch (rule.type) {
				case 'rule':
					const { selectors, ...rest } = rule;

					let globalize = false;
					let ignore = false;

					const result = {
						selectors: selectors.map(selector =>
							cssWhat.stringify(
								cssWhat.parse(selector).map(tokens =>
									tokens
										.map(t => {
											if (
												t.type === 'tag' &&
												t.name === '@global'
											) {
												globalize = true;
												return false;
											} else if (
												t.type === 'attribute' &&
												t.name === 'class'
											) {
												const newValue = globalize
													? t.value
													: `${t.value}-${uid}`;
												mappings[
													camelize(t.value)
												] = newValue;
												return {
													...t,
													value: newValue
												};
											} else if (
												t.type === 'tag' &&
												t.name === '@meta'
											) {
												// Don't emit it in the resulting CSS.
												ignore = true;

												for (const decl of rule.declarations) {
													switch (decl.property) {
														case 'priority':
															priority = parseFloat(
																decl.value,
																10
															);
															break;
													}
												}
											}

											return t;
										})
										.filter(Boolean)
								)
							)
						),
						...rest
					};

					return ignore ? null : result;
				default:
					return rule;
			}
		})
		.filter(Boolean);

	return {
		content: css.stringify(ast),
		priority
	};
}

export default () => ({
	name: 'css-modules',
	setup(build) {
		const cssFiles = new Map();
		const cssFilesList = [];

		build.onLoad({ filter: /\.css$/ }, async args => {
			const contents = await fsp.readFile(args.path, 'utf-8');

			let mappings = {};

			if (cssFiles.has(args.path)) {
				({ mappings } = cssFiles.get(args.path));
			} else {
				const { content, priority } = await processCSS(
					contents,
					cssFiles.size,
					mappings
				);

				cssFiles.set(args.path, {
					content,
					mappings
				});

				cssFilesList.push({ pathname: args.path, priority });
			}

			return {
				contents: Object.entries(mappings)
					.map(e => `export const ${e[0]} = ${JSON.stringify(e[1])};`)
					.join('\n'),
				loader: 'js'
			};
		});

		build.onEnd(result => {
			if (result.outputFiles.length === 0) {
				// We need to know where to put it based on its sibling output files.
				console.warn(
					'WARNING: not emitting a .css file because it appears there are no other output files (or an error occurred somewhere else in the build)'
				);
				return result;
			}

			const buildDir = path.dirname(result.outputFiles[0].path);

			cssFilesList.sort((a, b) => a.priority - b.priority);

			const chunks = [];

			for (const { pathname } of cssFilesList) {
				const cssFile = cssFiles.get(pathname);
				chunks.push(`/* ${path.relative(buildDir, pathname)} */`);
				chunks.push(cssFile.content);
				chunks.push('');
			}

			let outText = chunks.join('\n');

			if (build.initialOptions.minify) {
				outText = css.stringify(css.parse(outText), { compress: true });
			}

			// No, you really cannot .push(). ESBuild's bindings don't allow for it.
			// Not really upset by it, but I realize it looks weird.
			result.outputFiles = [
				...result.outputFiles,
				{
					path: path.join(
						buildDir,
						path.basename(
							build.initialOptions.outfile,
							path.extname(build.initialOptions.outfile)
						) + '.css'
					),
					text: outText,
					contents: toArrayBuffer(outText)
				}
			];

			return result;
		});
	}
});
