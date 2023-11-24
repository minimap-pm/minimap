import S from 's-js';

const overrideSymbol = Symbol('execute-override');

export default ({ width, height, offsetX, offsetY } = {}) => elem =>
	S(() => {
		const handler = e => {
			if (e !== overrideSymbol && e?.target !== window) return;

			S.freeze(() => {
				width?.(elem.clientWidth);
				height?.(elem.clientHeight);
				offsetX?.(elem.offsetLeft);
				offsetY?.(elem.offsetTop);
			});
		};

		window.addEventListener('resize', handler);
		const interval = setInterval(handler, 100, overrideSymbol);

		S.cleanup(() => {
			window.removeEventListener('resize', handler);
			clearInterval(interval);
		});

		handler(overrideSymbol);
	});
