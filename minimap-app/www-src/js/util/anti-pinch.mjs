/*
	The web is fucking broken.
	Dennis Ritchie is rolling in his grave.
	The world is completely fucked.

	Call this on `document.body` somewhere
	up top in index.mjs if you want to use it.
*/

export default elem => {
	let realCtrlKey = false;
	elem.addEventListener('keydown', e => {
		if (e.key === 'Control') realCtrlKey = true;
	});
	elem.addEventListener('keyup', e => {
		if (e.key === 'Control') realCtrlKey = false;
	});

	elem.addEventListener(
		'wheel',
		e => {
			if (!realCtrlKey && e.ctrlKey) {
				e.preventDefault();
				return false;
			}
		},
		{ passive: false }
	);
};
