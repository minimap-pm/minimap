import { transform as transformSucrase } from '@qix/sucrase';

const toArrayBuffer = text => new TextEncoder('utf-8').encode(text);

export default () => ({
	name: 'sucrase',
	setup(build) {
		build.onEnd(result => {
			result.outputFiles = result.outputFiles.map(
				({ text, path, contents }) => {
					if (path.match(/\.m?js$/)) {
						const newCode = transformSucrase(text, {
							transforms: []
						}).code;

						return {
							text: newCode,
							path,
							contents: toArrayBuffer(newCode)
						};
					}

					return { text, path, contents };
				}
			);

			return result;
		});
	}
});
