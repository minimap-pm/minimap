import { promises as fsp } from 'fs';

import css from 'css';

export default async function getThemeList(filename) {
	const contents = await fsp.readFile(filename, 'utf-8');

	return css
		.parse(contents)
		?.stylesheet?.rules?.filter(r => r.type === 'rule')
		.reduce((acc, r) => {
			if (r.selectors.length > 0) acc.push(...r.selectors);
			return acc;
		}, [])
		.map(sel => sel.match(/^(?:@\w+ +)*body\.mm\.theme\.(\w+)$/)?.[1])
		.filter(Boolean);
}
