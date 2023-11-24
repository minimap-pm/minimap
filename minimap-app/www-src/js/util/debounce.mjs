import S from 's-js';

export default (signal, { timeout = 100, factory = S.value } = {}) => {
	const result = factory(S.sample(signal));

	let throttled = false;
	let hasScheduled = false;
	let scheduled;

	const execute = () => {
		if (hasScheduled) {
			hasScheduled = false;
			setTimeout(execute, timeout);
			result(scheduled);
		} else {
			throttled = false;
		}
	};

	S.on(
		signal,
		() => {
			if (throttled) {
				hasScheduled = true;
				scheduled = S.sample(signal);
			} else {
				throttled = true;
				setTimeout(execute, timeout);
				result(S.sample(signal));
			}
		},
		null,
		true
	);

	return result;
};
