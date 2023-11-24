import fs from 'fs';

import { readTSV, TSVColumns } from './tsv.mjs';

/* MUST be synchronous */
export default (tsvFile, { defaultLangs = [] } = {}) => {
	const contents = fs.readFileSync(tsvFile, 'utf-8');
	const corpus = readTSV(contents, 'key');
	return [...defaultLangs, ...corpus[TSVColumns]];
};
