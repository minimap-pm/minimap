import * as Surplus from 'surplus';

import css from 'minimap/js/util/css.mjs';

import * as C from './Spacer.css';

export default ({ className }) => (
	<div className={css(C.root, className)}>&nbsp;</div>
);
