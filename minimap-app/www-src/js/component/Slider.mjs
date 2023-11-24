import * as Surplus from 'surplus';
import S from 's-js';
import data from 'surplus-mixin-data';

import sig from 'minimap/js/util/sig.mjs';
import css from 'minimap/js/util/css.mjs';
import uniqueClass from 'minimap/js/util/unique-class.mjs';

import * as C from './Slider.css';
import * as CG from '../global.css';

export default ({
	editable = true,
	min = 0,
	max = 100,
	value,
	label,
	className,
	...props
}) => {
	let scrubber;
	const id = uniqueClass('mm-slider');

	const elem = (
		<div className={css(C.root, CG.focusWithinOutline, className)}>
			{label && (
				<label className={C.label} for={id}>
					{sig(label)}
				</label>
			)}
			<input
				className={C.text}
				name={editable ? id : ''}
				id={editable ? id : ''}
				type="number"
				min={sig(min)}
				max={sig(max)}
				fn={data(value)}
				disabled={!editable}
			/>
			<div className={C.scrubberWrapper}>
				<input
					type="range"
					id={!editable ? id : ''}
					name={!editable ? id : ''}
					ref={scrubber}
					min={sig(min)}
					max={sig(max)}
					fn={data(value)}
					{...props}
				/>
			</div>
		</div>
	);

	S(() => {
		const cursor = ((value() - min) / (max - min)) * 100;
		scrubber.style.background = `linear-gradient(to right, var(--mm-text) 0%, var(--mm-text) ${cursor}%, var(--mm-border) ${cursor}%, var(--mm-border) 100%)`;
	});

	return elem;
};
