const makePromise = () => {
	let res, rej;
	const p = new Promise((_res, _rej) => {
		res = _res;
		rej = _rej;
	});

	return {
		promise: p,
		resolve: res,
		reject: rej
	};
};

const passthrough = v => v;

makePromise.on = (obj, eventName, transform = passthrough) => {
	const { promise, resolve, reject } = makePromise();
	let handled = false;

	obj.once(eventName, (...args) => {
		if (handled) return;
		handled = true;

		try {
			resolve(transform(...args));
		} catch (error) {
			reject(error);
		}
	});

	obj.once('error', error => {
		if (handled) return;
		handled = true;
		reject(error);
	});

	return promise;
};

export default makePromise;
