import S from 's-js';

import sig from 'minimap/js/util/sig.mjs';

export default disabled => elem =>
	S(() => {
		if (sig(disabled)) {
			elem.setAttribute('disabled', '1');
		} else {
			elem.removeAttribute('disabled');
		}
	});
