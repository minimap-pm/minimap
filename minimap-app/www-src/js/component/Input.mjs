import * as Surplus from 'surplus';
import data from 'surplus-mixin-data';

import sig from '@minimap/web/js/util/sig.mjs';
import css from '@minimap/web/js/util/css.mjs';
import disable from '@minimap/web/js/util/disable.mjs';

import * as C from './Input.css';

const TextArea = ({ children, className, ...props }) => (
	<textarea className={className()} {...props}>
		{children}
	</textarea>
);

const Input = ({ children, className, ...props }) => (
	<input className={className()} {...props}>
		{children}
	</input>
);

export default ({
	value,
	disabled = false,
	placeholder = '',
	type = 'text',
	invalid,
	className,
	...props
}) => {
	type = sig(type);

	const ElemType = type === 'textarea' ? TextArea : Input;

	// XXX The weird `fn={...}` attribute is due to a bug in ESBuild.
	return (
		<ElemType
			fn={elem => (data(value)(elem), disable(disabled)(elem))}
			placeholder={sig(placeholder)}
			className={() => css(C.root, sig(invalid) && C.invalid, className)}
			{...(type !== 'textarea' && { type })}
			{...props}
		/>
	);
};
