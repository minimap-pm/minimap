export default constsMap => ({
	name: 'consts',
	setup(build) {
		build.onResolve({ filter: /^consts:\w+$/ }, args => ({
			path: args.path,
			namespace: 'consts-ns'
		}));

		build.onLoad({ filter: /.*/, namespace: 'consts-ns' }, args => {
			const key = args.path.match(/^consts:(\w+)$/)[1];

			if (!(key in constsMap)) {
				throw new Error(
					`'consts:${key}' import is invalid; '${key}' key does not exist in config map`
				);
			}

			return {
				contents: JSON.stringify(constsMap[key]),
				loader: 'json'
			};
		});
	}
});
