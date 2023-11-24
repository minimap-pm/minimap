import S from 's-js';

import sevent from 'minimap/js/util/s-event.mjs';
import sig from 'minimap/js/util/sig.mjs';

export default (onUpdate, editable = null) => elem =>
	S(() => {
		let ignore = false;
		let lastKnown = elem.innerText;

		sevent.dom(elem, 'keydown', ev => {
			if (ev.keyCode === 13) {
				// enter
				ev.preventDefault();
				ev.stopPropagation();

				ev.target.blur();

				return false;
			} else if (ev.keyCode === 27) {
				// escape
				ev.preventDefault();
				ev.stopPropagation();
				ev.target.innerText = lastKnown;
				ignore = true;
				ev.target.blur();
				return false;
			}
		});

		sevent.dom(elem, 'focus', ev => {
			lastKnown = ev.target.innerText;
		});

		sevent.dom(elem, 'blur', ev => {
			if (ignore) {
				ignore = false;
				return;
			}

			if (lastKnown === ev.target.innerText) {
				// nothing to do.
				return;
			}

			const prevKnown = lastKnown;
			lastKnown = ev.target.innerText;

			try {
				onUpdate?.(ev.target.innerText.replace(/\r?\n/g, ''));
			} catch (err) {
				// fall back
				lastKnown = prevKnown;
				ev.target.innerText = prevKnown;
				throw err;
			}
		});

		S(() =>
			elem.setAttribute(
				'contenteditable',
				editable === null || sig(editable) ? 'true' : 'false'
			)
		);
	});
