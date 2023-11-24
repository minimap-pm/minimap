import * as Surplus from 'surplus';

import prevent from 'minimap/js/util/prevent.mjs';

export default ({ children, onSubmit, ...props }) => (
	<form
		onSubmit={e => {
			prevent(e);
			onSubmit?.(e);
			return false;
		}}
		{...props}
	>
		{children}
	</form>
);
