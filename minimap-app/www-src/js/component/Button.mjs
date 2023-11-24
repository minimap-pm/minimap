import * as Surplus from 'surplus';

import sig from 'minimap/js/util/sig.mjs';
import css from 'minimap/js/util/css.mjs';
import disable from 'minimap/js/util/disable.mjs';

import Link from 'minimap/js/component/Link.mjs';

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

export const LinkButton = ({
	disabled = false,
	className,
	children,
	...props
}) => (
	<Link
		className={css(C.root, C.link, className, sig(disabled) && C.disabled)}
		{...props}
	>
		{children}
	</Link>
);
