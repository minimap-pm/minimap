const PATTERN = /\\(?:([rnvft"'`\\bae])|x([a-fA-F0-9]{2})|u([a-fA-F0-9]{4}))/g;

const CHARS = {
	r: '\r',
	n: '\n',
	v: '\v',
	f: '\f',
	t: '\t',
	'"': '"',
	"'": "'",
	'`': '`',
	'\\': '\\',
	b: '\b',
	a: 'a',
	e: '\x1b'
};

export default str =>
	str.replace(PATTERN, (_, char, hex, unicode) =>
		char ? CHARS[char] : String.fromCharCode(parseInt(hex ?? unicode, 16))
	);
