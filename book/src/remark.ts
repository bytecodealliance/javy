import { visit, SKIP } from 'unist-util-visit';
import type { Root } from 'mdast';
import type { VFile } from 'vfile';
import { docSlug } from './slugs';

const fromDocs = (file: VFile) => /\/docs\/docs-[^/]+\.md$/.test(file.path ?? '');

/**
 * Starlight renders the page title itself, so drop the heading the docs open
 * with (it became the title, see `content.config.ts`).
 */
export function remarkStripLeadingHeading() {
	return (tree: Root, file: VFile) => {
		if (!fromDocs(file)) return;
		const first = tree.children.findIndex((node) => node.type !== 'yaml');
		if (tree.children[first]?.type === 'heading') tree.children.splice(first, 1);
	};
}

/**
 * Rewrites links between docs (`./docs-using-nodejs.md#steps`) to book routes
 * (`/javy/using/nodejs/#steps`), links to `index.md` to the home page, and
 * links to the rest of the repository (`../wpt/README.md`) to GitHub.
 */
export function remarkDocsLinks({ base, repo }: { base: string; repo: string }) {
	const root = base.endsWith('/') ? base : `${base}/`;
	return (tree: Root, file: VFile) => {
		if (!fromDocs(file)) return;
		visit(tree, 'link', (node) => {
			if (node.url.startsWith('../')) {
				node.url = new URL(node.url, `${repo}/blob/main/docs/`).href;
				return;
			}
			const match = node.url.match(/^(?:\.\/)?((?:docs-[\w-]+|index)(?:\.md)?)(#.*)?$/);
			if (!match) return;
			const [, target, hash = ''] = match;
			const slug = target.startsWith('index') ? '' : `${docSlug(target)}/`;
			node.url = `${root}${slug}${hash}`;
		});
	};
}

/**
 * Turns ```mermaid fences into `<pre class="mermaid">` so they skip syntax
 * highlighting and are rendered client-side by `src/mermaid.js`.
 */
export function remarkMermaid() {
	return (tree: Root) => {
		visit(tree, 'code', (node, index, parent) => {
			if (node.lang !== 'mermaid' || !parent || index === undefined) return;
			const escaped = node.value.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
			parent.children[index] = { type: 'html', value: `<pre class="mermaid">${escaped}</pre>` };
			return SKIP;
		});
	};
}
