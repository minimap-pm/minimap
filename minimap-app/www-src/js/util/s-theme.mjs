import S from 's-js';

const theme = S.root(() =>
	S.value(
		window?.matchMedia?.('(prefers-color-scheme: light)').matches
			? 'light'
			: 'dark'
	)
);

window
	?.matchMedia?.('(prefers-color-scheme: light)')
	.addEventListener('change', e => theme(e.matches ? 'light' : 'dark'));

export default (...args) => {
	if (args.length > 0) {
		throw new Error('system theme S variable cannot be manually updated');
	}

	return theme();
};
