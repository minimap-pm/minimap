import * as Surplus from 'surplus';
import S from 's-js';

import * as C from './Root.css';

export default ({ view: requestedView, fadeTime = 75 }) => {
	const currentView = S.value(S.sample(requestedView));

	const rootElem = <div className={C.root}>{currentView()}</div>;

	rootElem.style.transition = `opacity ${fadeTime / 1000}s ease-out`;

	S(() => {
		if (requestedView() !== currentView()) {
			rootElem.style.opacity = '0';
			// should match opacity transition time in CSS
			setTimeout(() => {
				currentView(requestedView());
				rootElem.style.removeProperty('opacity');
			}, fadeTime);
		}
	});

	return rootElem;
};
