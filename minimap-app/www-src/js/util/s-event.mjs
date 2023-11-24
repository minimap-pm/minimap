import S from 's-js';

const sevent = (target, name, fn) => {
	target.on(name, fn);
	S.cleanup(() => target.off(name, fn));
};

sevent.dom = (target, name, fn) => {
	target.addEventListener(name, fn);
	S.cleanup(() => target.removeEventListener(name, fn));
};

export default sevent;
