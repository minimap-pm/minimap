import * as Surplus from 'surplus';

import I from 'minimap/js/util/i18n.mjs';
import sig from 'minimap/js/util/sig.mjs';

import * as C from './Loading.css';

export default ({ children }) => (
	<div className={C.root}>
		{children?.length > 0 ? children : I`loading...`}
	</div>
);
