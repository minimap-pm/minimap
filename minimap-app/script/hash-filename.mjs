import path from 'node:path';
import stream from 'node:stream';
import crypto from 'node:crypto';

function hashFileName(opts) {
	const transformer = new stream.Transform({ objectMode: true });
	if (!opts || typeof opts !== 'object' || !opts.dict) {
		throw new Error('must pass options with `dict` field');
	}

	transformer._transform = function (file, _, cb) {
		const hasher = crypto.createHash('sha256');
		hasher.update(file.contents);
		const hash = hasher.digest('hex');
		opts.dict[path.basename(file.path)] = hash;
		file.path = path.join(
			path.dirname(file.path),
			`${path.basename(
				file.path,
				path.extname(file.path)
			)}.${hash}${path.extname(file.path)}`
		);
		this.push(file);
		cb();
	};

	return transformer;
}

export default hashFileName;
