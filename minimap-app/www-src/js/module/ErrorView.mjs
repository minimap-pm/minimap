import * as Surplus from 'surplus';

import I from 'minimap/js/util/i18n.mjs';
import css from 'minimap/js/util/css.mjs';
import { Alert } from 'minimap/js/component/Icon.mjs';

import * as C from './ErrorView.css';

const jsonReplacer = (k, v) => {
	if (v instanceof Error) {
		return { exception: v.stack.split('\n') };
	}

	return v;
};

export default ({ className, errorMessage }) => (
	<div className={css(C.root, className)}>
		<h1>
			<span className={C.icon}>
				<Alert />
			</span>
			<span>{errorMessage().message}</span>
		</h1>
		<div>
			{I`A fatal error occurred. It is most likely an incompatibility with your browser. Below is extra context that can be sent to us in the event you'd like to report a problem or for support.`}
		</div>
		<div>
			{I`Make sure you're using an updated "evergreen" browser, such as Firefox, Safari or Chrome.`}
		</div>
		{/* prettier-ignore */}
		<pre><code>{
			errorMessage().context
				? JSON.stringify(errorMessage().context, jsonReplacer, 2)
				: I`(no context was provided with this error message)`
		}</code></pre>
	</div>
);
