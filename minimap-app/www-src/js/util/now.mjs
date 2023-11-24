import S from 's-js';

export default (updateDelaySeconds = 1) => {
	updateDelaySeconds = Math.max(Number(updateDelaySeconds), 1);
	const currentDate = S.value(new Date());
	const handle = setInterval(
		() => currentDate(new Date()),
		updateDelaySeconds * 1000
	);
	S.cleanup(() => clearInterval(handle));
	return currentDate;
};
