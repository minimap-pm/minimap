// Overengineering? We don't know her.
export default (strs, args) => {
	let i = strs.length + args.length;
	const res = new Array(i);
	const a = [strs, args];
	while (--i >= 0) res[i] = a[i % 2][i >> 1];
	return res;
};
