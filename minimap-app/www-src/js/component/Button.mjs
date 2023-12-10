import * as Surplus from 'surplus';

import sig from 'minimap/js/util/sig.mjs';
import css from 'minimap/js/util/css.mjs';
import disable from 'minimap/js/util/disable.mjs';

import * as C from './Button.css';

export default ({
	type = 'button',
	disabled = false,
	className,
	children,
	...props
}) => (
	<button
		className={css(C.root, className)}
		type={type}
		fn={disable(disabled)}
		{...props}
	>
		{children}
	</button>
);
