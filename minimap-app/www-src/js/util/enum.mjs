const immutable = v => ({
	value: v,
	enumerable: false,
	configurable: false,
	writable: false
});

export default (...constants) =>
	constants.reduce(
		(acc, k, i) => ((acc[k] = i), acc),
		Object.defineProperties(
			{},
			{
				nameSet: immutable(new Set(constants)),
				names: immutable(constants.slice())
			}
		)
	);
