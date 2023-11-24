const TERMINAL = Symbol('@');

function* splitPath(str) {
	const splits = str.replace(/^\/+|\/+$/g, '').split(/\/+/g);
	for (const split of splits) if (split) yield split;
}

function* zipPath(strs, args) {
	if (strs[0]) yield* splitPath(strs[0]);
	const len = strs.length - 1;
	for (let i = 0; i < len; ) {
		yield args[i];
		// test here since `foo${bar}` results in a dangling empty string,
		// and we don't want to consider the empty space between two
		// args.
		const s = strs[++i];
		if (s) yield* splitPath(s);
	}
}

function* traverseLeafs(leafs) {
	let iterator;

	// array arguments means "these are already separated"
	if (Array.isArray(leafs)) {
		iterator = leafs;
	} else if (typeof leafs === 'string') {
		iterator = splitPath(leafs);
	} else {
		throw new TypeError('argument must be a string or array');
	}

	for (const leaf of iterator) {
		yield decodeURIComponent(leaf.toString());
	}
}

function matchAndTransform(arg, pattern) {
	const m = arg.match(pattern);
	return Boolean(m) ? (pattern.sticky ? m : m[0]) : undefined;
}

function traverseBranch(leafs, from, branch, pack) {
	const handler = branch.get(TERMINAL, null);

	if (from >= leafs.length) {
		return handler ? ctx => handler(ctx, ...pack) : undefined;
	}

	const nestedPack = handler ? [] : pack;

	const leaf = leafs[from];
	++from;

	for (const [leafTest, nextBranch] of branch) {
		let nextPack = nestedPack;

		if (leafTest === TERMINAL) {
			continue;
		} else if (typeof leafTest === 'string') {
			if (leafTest !== leaf) continue;
		} else {
			const arg = leafTest(leaf);
			if (!arg) continue;
			nextPack = [...nextPack, arg];
		}

		const nestedHandler = traverseBranch(leafs, from, nextBranch, nextPack);

		if (nestedHandler !== undefined) {
			return handler
				? ctx => {
						const res = nestedHandler(ctx);
						return res !== true ||
							typeof res === 'string' ||
							Array.isArray(res)
							? res
							: handler(ctx, ...pack);
				  }
				: nestedHandler;
		}
	}
}

const makeRouter = onRoute => {
	// The use of `Map` here is important - order of insertion
	// is required to be preserved here for this to work.
	const rootTrie = new Map();

	const tagRoute = root => (strs, ...args) => handler => {
		let branch = root;

		for (let leaf of zipPath(strs, args)) {
			if (leaf instanceof RegExp) {
				const rex = leaf;
				leaf = arg => matchAndTransform(arg, rex);
			} else if (
				!(typeof leaf === 'function' || typeof leaf === 'string')
			) {
				throw new TypeError(
					'leaf must either be a string, regular expression, or function'
				);
			}

			const nextBranch = branch.get(leaf, null);
			if (nextBranch) {
				branch = nextBranch;
			} else {
				const m = new Map();
				branch.set(leaf, m);
				branch = m;
			}
		}

		if (typeof handler !== 'function') {
			throw new TypeError('handler must be a function');
		}

		if (branch.has(TERMINAL)) {
			throw new Error('handler already defined for this branch');
		}

		branch.set(TERMINAL, handler);

		return tagRoute(branch);
	};

	const rootRouter = tagRoute(rootTrie);

	let currentRoute = null;

	rootRouter.route = (arg, routeState) => {
		const leafs = [...traverseLeafs(arg)];
		const handler = traverseBranch(leafs, 0, rootTrie, []);

		const encodedPath = '/' + leafs.map(encodeURIComponent).join('/');

		if (handler) {
			const ctx = { routeState };
			const routeResult = handler(ctx);

			if (typeof routeResult === 'string' || Array.isArray(routeResult)) {
				// redirection request
				// use the routeState on the context so that a redirection
				// can dictate how the redirect affects route state.
				return rootRouter.route(routeResult, ctx.routeState);
			}

			currentRoute = encodedPath;
			onRoute(ctx, encodedPath);

			return true;
		} else {
			// Route not found
			onRoute(null, encodedPath);
		}

		return false;
	};

	return rootRouter;
};

makeRouter.attach = (onRoute, docWindow = window) => {
	const router = makeRouter((ctx, pathname) => {
		const newUrl = new URL(docWindow.location);
		newUrl.pathname = pathname;

		const res = onRoute(ctx, pathname);

		if (ctx !== null) {
			switch (ctx.routeState) {
				case 'ignore':
					break;
				case 'replace':
					history.replaceState({}, '', newUrl);
					break;
				default:
					history.pushState({}, '', newUrl);
					break;
			}
		}

		return res;
	});

	docWindow.addEventListener('popstate', () =>
		router.route(docWindow.location.pathname, 'ignore')
	);

	router.init = () => router.route(docWindow.location.pathname, 'ignore');

	return router;
};

export default makeRouter;
