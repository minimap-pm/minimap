import S from 's-js';

export const regexFilter = (
	signal,
	pattern,
	{ spaceReplace = false } = {}
) => elem => {
	let pSource = `(?<_f_chr>${pattern.source})`;
	let pFlags = pattern.flags;

	if (!pattern.global) pFlags += 'g';

	if (typeof spaceReplace === 'string') {
		pSource += String.raw`|(?<_f_ws>[ \r\n\t\v\f])`;
	}

	pattern = new RegExp(pSource, pFlags);

	let isUpdating = false;

	const updateValue = v => {
		if (isUpdating) return;

		try {
			isUpdating = true;

			const chunks = [];

			let m;
			while ((m = pattern.exec(v))) {
				chunks.push(
					m.groups['_f_ws'] === undefined
						? m.groups['_f_chr']
						: spaceReplace
				);
			}

			const newValue = chunks.join('');

			signal(newValue);
			elem.value = newValue;
		} finally {
			isUpdating = false;
		}
	};

	const update = e => {
		e.preventDefault();
		e.stopPropagation();
		updateValue(e.target.value);
		return false;
	};

	elem.addEventListener('input', update);
	S.cleanup(() => elem.removeEventListener('input', update));

	S(() => updateValue(signal()));
};
