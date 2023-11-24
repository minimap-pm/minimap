import S from 's-js';

import appConfig from 'consts:appConfig';

import makeId from 'minimap/js/util/i18n-id.mjs';

const defaultLanguageSet = new Set(appConfig.defaultLanguages || []);

function getBrowserLanguage() {
	if (!appConfig.supportedLanguages) {
		console.warn('app constants are missing "supportedLanguages"');
		return;
	}

	const browserLangList =
		navigator?.languages || (navigator?.language && [navigator.language]);

	if (!browserLangList) return;

	const supportedLangSet = new Set(appConfig.supportedLanguages);

	for (const browserLang of browserLangList) {
		if (supportedLangSet.has(browserLang)) return browserLang;
	}
}

const [
	corpus,
	lang,
	relativeTimeFormatter,
	relativeTimeFormatterNarrow,
	absoluteTimeFormatter
] = S.root(() => {
	const corpus = S.value(null);
	const lang = S.value(getBrowserLanguage());

	const loadedCorpora = new Map();

	S(() => {
		console.log('language set to', lang());

		/*
			Check the `defaultLanguageSet` here since the default
			languages are built into the app itself and thus require
			no fetching to occur. This sets the corpus to null, which
			hits a different codepath in the internationalization
			helper (I`...`) and returns the given string directly.
		*/
		if (lang() && !defaultLanguageSet.has(lang())) {
			if (loadedCorpora.has(lang())) {
				corpus(loadedCorpora.get(lang()));
			} else if (lang().match(/^[\w_-]+$/i)) {
				const requestedLang = lang();

				fetch(`lang/${lang()}.json`)
					.then(response => response.json())
					.then(data => {
						const newCorpus = data;
						loadedCorpora.set(requestedLang, newCorpus);

						/* Make sure we don't have race conditions */
						if (lang() === requestedLang) {
							console.debug('loaded corpus for', requestedLang);
							corpus(newCorpus);
						}
					})
					.catch(err => {
						console.error(
							'failed to load corpus:',
							requestedLang,
							err
						);
					});
			} else {
				console.error('invalid lang:', lang());
			}
		} else {
			/* Use default corpus */
			corpus(null);
		}
	});

	const relativeTimeFormatter = S(
		() =>
			new Intl.RelativeTimeFormat(lang() ?? 'en', {
				numeric: 'auto',
				style: 'long'
			})
	);

	const relativeTimeFormatterNarrow = S(
		() =>
			new Intl.RelativeTimeFormat(lang() ?? 'en', {
				numeric: 'always',
				style: 'narrow'
			})
	);

	const absoluteTimeFormatter = S(
		() =>
			new Intl.DateTimeFormat(lang() ?? 'en', {
				dateStyle: 'full',
				timeStyle: 'long'
			})
	);

	return [
		corpus,
		lang,
		relativeTimeFormatter,
		relativeTimeFormatterNarrow,
		absoluteTimeFormatter
	];
});

const I = (strings, ...args) => {
	if (corpus()) {
		const id = makeId(strings.raw);
		let trail = corpus()[id];

		if (trail) {
			trail = trail.slice();
			for (let i = 1, len = trail.length; i < len; i += 2) {
				trail[i] = args[trail[i]];
			}
			return trail.join('');
		} else {
			console.warn('lang', S.sample(lang), 'missing translation:', id);
		}
	}

	return String.raw({ raw: strings }, ...args);
};

const relativeFormatWith = (ms, formatter) => {
	ms /= 1000;
	let a = Math.abs(ms);

	// now!
	if (a < 5) return I`now`;
	// seconds (2 minute threshold)
	if (a < 120) return formatter.format(Math.trunc(ms), 'second');
	// minutes
	if (a < 3600) return formatter.format(Math.trunc(ms / 60), 'minute');
	// hours
	if (a < 86400) return formatter.format(Math.trunc(ms / 3600), 'hour');
	// days
	if (a < 604800) return formatter.format(Math.trunc(ms / 86400), 'day');
	// weeks
	if (a < 2678400) return formatter.format(Math.trunc(ms / 604800), 'week');
	// months
	if (a < 31536000)
		return formatter.format(Math.trunc(ms / 2678400), 'month');
	// years
	return formatter.format(Math.trunc(ms / 31536000), 'month');
};

I.language = lang;
I.date = (d, opts) => d.toLocaleDateString(I.language() ?? 'en', opts);
I.time = (d, opts) => d.toLocaleTimeString(I.language() ?? 'en', opts);
I.fullDateTime = d => absoluteTimeFormatter()?.format(d) ?? '';
I.relative = ms =>
	relativeTimeFormatter()
		? relativeFormatWith(ms, relativeTimeFormatter())
		: '';
I.relativeNarrow = ms =>
	relativeTimeFormatterNarrow()
		? relativeFormatWith(ms, relativeTimeFormatterNarrow())
		: '';

export default I;
