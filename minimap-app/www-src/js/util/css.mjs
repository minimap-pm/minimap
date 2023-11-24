import S from 's-js';

import sig from 'minimap/js/util/sig.mjs';

const css = (...classes) =>
	classes
		.map(v => sig(v))
		.filter(Boolean)
		.join(' ');

css.vars = vars => elem =>
	S(() => {
		for (const name of Object.keys(vars)) {
			elem.style.setProperty(`--${name}`, sig(vars[name]).toString());
		}
	});

export default css;
