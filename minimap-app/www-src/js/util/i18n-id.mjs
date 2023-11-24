export default strings => {
	const chunks = [strings[0]];

	for (let i = 0, len = strings.length - 1; i < len; i++) {
		chunks.push(`{${i}}`, strings[i + 1]);
	}

	return chunks.join('');
};
