// Render ```mermaid diagrams from the docs (see `remarkMermaid`) only on
// pages that have them, and re-render when the theme changes.
const diagrams = [...document.querySelectorAll('pre.mermaid')];
if (diagrams.length > 0) {
	const sources = diagrams.map((el) => el.textContent);
	const { default: mermaid } = await import(
		'https://cdn.jsdelivr.net/npm/mermaid@11.4.1/dist/mermaid.esm.min.mjs'
	);
	const render = async () => {
		const dark = document.documentElement.dataset.theme === 'dark';
		mermaid.initialize({ startOnLoad: false, theme: dark ? 'dark' : 'neutral' });
		diagrams.forEach((el, i) => {
			el.removeAttribute('data-processed');
			el.textContent = sources[i];
		});
		await mermaid.run({ nodes: diagrams });
	};
	await render();
	new MutationObserver(render).observe(document.documentElement, {
		attributeFilter: ['data-theme'],
	});
}
