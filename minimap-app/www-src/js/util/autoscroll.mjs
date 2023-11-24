import S from 's-js';

export default elem =>
	S(() => {
		const scrollTop = S.value(0);
		const scrollHeight = S.value(0);
		const clientHeight = S.value(0);
		const autoScroll = S.value(true);

		S(() => autoScroll(scrollTop() + clientHeight() >= scrollHeight()));

		const updateMetrics = () =>
			S.freeze(() => {
				scrollTop(elem.scrollTop);
				scrollHeight(elem.scrollHeight);
				clientHeight(elem.clientHeight);
			});

		elem.addEventListener('scroll', updateMetrics);

		const observer = new MutationObserver(() => {
			if (autoScroll()) {
				elem.scrollTop = elem.scrollHeight;
			}

			updateMetrics();
		});

		observer.observe(elem, { childList: true, subtree: true });

		S.cleanup(() => observer.disconnect());
	});
