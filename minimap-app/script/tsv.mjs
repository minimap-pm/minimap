export const TSVColumns = Symbol('TSVColumns');

export const readTSV = (contents, key) => {
	const corpus = {};

	const lines = contents
		.replace(/^(\r?\n)*|(\r?\n)*$/g, '')
		.split(/(?:\r?\n)+/g)
		.filter(line => line.match(/^\t*[^\t].*$/));

	let header = lines.shift();
	if (header) {
		header = header.split('\t');

		{
			const symbolicHeader = new Set(header);
			symbolicHeader.delete(key);
			corpus[TSVColumns] = symbolicHeader;
		}

		lines.forEach(line => {
			const vals = line.split('\t');
			const record = {};

			for (
				let i = 0, len = Math.min(header.length, vals.length);
				i < len;
				i++
			) {
				record[header[i]] = vals[i];
			}

			if (!record[key]) return;
			corpus[record[key]] = record;
			delete record[key];
		});
	}

	return corpus;
};

export const writeTSV = (obj, key) => {
	const allKeys = new Set();

	const recordKeys = Object.keys(obj);
	recordKeys.sort();

	const records = [];
	for (const recordKey of recordKeys) {
		const record = obj[recordKey];
		for (const k of Object.keys(record)) allKeys.add(k);
		records.push({ [key]: recordKey, ...record });
	}

	const keyList = [...allKeys];
	keyList.sort();
	keyList.unshift(key);

	const rows = [keyList];

	for (const record of records) {
		const row = new Array(keyList.length).fill('');

		for (let i = 0, len = keyList.length; i < len; i++) {
			row[i] = record[keyList[i]] || '';
		}

		rows.push(row);
	}

	return rows.map(row => row.join('\t')).join('\n') + '\n';
};
