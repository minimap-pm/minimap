import S from 's-js';

/*
	Use this utility whenever you want
	to cache a display value even if the
	original value itself is unset.

	Useful for fade-out transitions
	and whatnot where otherwise the text
	would disappear right before fading out.

	You can also pass a delay to unset the
	cache value after a certain millisecond
	delay, too.
*/

export default (sig, delay) => {
	const cache = S.value(S.sample(sig));
	S(() => {
		if (sig()) {
			cache(sig());
		} else if (delay) {
			const handle = setTimeout(cache, delay, sig());
			S.cleanup(() => clearTimeout(handle));
		}
	});
	return () => cache(); // read only
};
