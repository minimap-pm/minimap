/*
	Originally by @kwdowik on Github:
	https://github.com/kwdowik/zoom-pan

	MIT License

	Modified by Josh Junon.
*/

import S from 's-js';
import sevent from 'minimap/js/util/s-event.mjs';
import sig from 'minimap/js/util/sig.mjs';

const hasPositionChanged = ({ pos, prevPos }) => pos !== prevPos;

const valueInRange = ({ minScale, maxScale, scale }) =>
	scale <= maxScale && scale >= minScale;

const getTranslate = ({ minScale, maxScale, scale }) => ({
	pos,
	prevPos,
	translate
}) =>
	valueInRange({ minScale, maxScale, scale }) &&
	hasPositionChanged({ pos, prevPos })
		? translate + (pos - prevPos * scale) * (1 - 1 / scale)
		: translate;

const getScale = ({
	scale,
	minScale,
	maxScale,
	scaleSensitivity,
	deltaScale
}) => {
	let newScale = scale + deltaScale / (scaleSensitivity / scale);
	newScale = Math.max(minScale, Math.min(newScale, maxScale));
	return [scale, newScale];
};

const getMatrix = ({ scale, translateX, translateY }) =>
	`matrix(${scale}, 0, 0, ${scale}, ${translateX}, ${translateY})`;

const pan = ({ state, originX, originY }) => {
	state.transformation.translateX += originX;
	state.transformation.translateY += originY;
	state.element.style.transform = getMatrix({
		scale: state.transformation.scale,
		translateX: state.transformation.translateX,
		translateY: state.transformation.translateY
	});
};

const makePan = state => ({
	panBy: ({ originX, originY }) => pan({ state, originX, originY }),
	panTo: ({ originX, originY, scale }) => {
		state.transformation.scale = scale;
		pan({
			state,
			originX: originX - state.transformation.translateX,
			originY: originY - state.transformation.translateY
		});
	}
});

const makeZoom = state => ({
	zoom: ({ x, y, deltaScale }) => {
		const { left, top } = state.element.getBoundingClientRect();
		const { minScale, maxScale, scaleSensitivity } = state;
		const [scale, newScale] = getScale({
			scale: state.transformation.scale,
			deltaScale,
			minScale,
			maxScale,
			scaleSensitivity
		});
		const originX = x - left;
		const originY = y - top;
		const newOriginX = originX / scale;
		const newOriginY = originY / scale;
		const translate = getTranslate({ scale, minScale, maxScale });
		const translateX = translate({
			pos: originX,
			prevPos: state.transformation.originX,
			translate: state.transformation.translateX
		});
		const translateY = translate({
			pos: originY,
			prevPos: state.transformation.originY,
			translate: state.transformation.translateY
		});

		state.element.style.transformOrigin = `${newOriginX}px ${newOriginY}px`;
		state.element.style.transform = getMatrix({
			scale: newScale,
			translateX,
			translateY
		});

		state.transformation.originX = newOriginX;
		state.transformation.originY = newOriginY;
		state.transformation.translateX = translateX;
		state.transformation.translateY = translateY;
		state.transformation.scale = newScale;
	}
});

export const makeController = ({
	element,
	minScale = 0.1,
	maxScale = 1,
	scaleSensitivity = 10
}) => {
	const state = {
		element,
		minScale,
		maxScale,
		scaleSensitivity,
		transformation: {
			originX: 0,
			originY: 0,
			translateX: 0,
			translateY: 0,
			scale: 1
		}
	};

	return { ...makeZoom(state), ...makePan(state), state };
};

export default (
	container,
	{ eventFilter, allowMulti = false, element, ...opts } = {}
) => {
	eventFilter ??= () => true;

	if (!allowMulti && container.children.length !== 1) {
		console.warn(
			'zoom-pan controller created on element with children.length !== 1; it will most certainly not function as expected.'
		);
	}

	const focusX = S.value(0);
	const focusY = S.value(0);

	// read-only
	const metrics = {
		originX: S.value(0),
		originY: S.value(0),
		translateX: S.value(0),
		translateY: S.value(0),
		scale: S.value(1),
		dragging: S.value(false)
	};

	element = element ?? container.children[0];

	const controller = makeController({
		element,
		...opts
	});

	const { transformation } = controller.state;
	let dragActive = false;

	const updateMetrics = () =>
		S.freeze(() => {
			metrics.originX(transformation.originX);
			metrics.originY(transformation.originY);
			metrics.translateX(transformation.translateX);
			metrics.translateY(transformation.translateY);
			metrics.scale(transformation.scale);
			metrics.dragging(dragActive);
		});

	sevent.dom(container, 'wheel', event => {
		if (eventFilter(event)) {
			controller.zoom({
				deltaScale: -Math.sign(event.deltaY),
				x: event.pageX,
				y: event.pageY
			});

			updateMetrics();
		}

		event.preventDefault();
		event.stopPropagation();

		return false;
	});

	sevent.dom(container, 'dblclick', () => {
		if (eventFilter(event)) {
			controller.panTo({
				originX: S.sample(focusX),
				originY: S.sample(focusY),
				scale: 1
			});

			updateMetrics();
		}

		event.preventDefault();
		event.stopPropagation();

		return false;
	});

	sevent.dom(document, 'mousemove', event => {
		if (!dragActive) return;

		controller.panBy({
			originX: event.movementX,
			originY: event.movementY
		});

		updateMetrics();

		event.preventDefault();
		event.stopPropagation();

		return false;
	});

	sevent.dom(container, 'mousedown', event => {
		if (eventFilter(event)) {
			dragActive = true;
			updateMetrics();
		}

		event.preventDefault();
		event.stopPropagation();

		return false;
	});

	sevent.dom(document, 'mouseup', event => {
		if (!dragActive) return;

		dragActive = false;
		updateMetrics();

		event.preventDefault();
		event.stopPropagation();

		return false;
	});

	sevent.dom(document, 'mouseleave', event => {
		if (
			event.clientY <= 0 ||
			event.clientX <= 0 ||
			event.clientX >= window.innerWidth ||
			event.clientY >= window.innerHeight
		) {
			dragActive = false;
			updateMetrics();
		}
	});

	return {
		focusX,
		focusY,
		metrics,
		zoomOut() {
			controller.zoom({
				x: 0,
				y: 0,
				deltaScale: -Infinity
			});
			updateMetrics();
		}
	};
};
