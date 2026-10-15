// @ts-check
import { readFileSync } from 'node:fs';
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import { remarkDocsLinks, remarkMermaid, remarkStripLeadingHeading } from './src/remark.ts';
import { tableOfContents, unlistedSlugs } from './src/toc.ts';

// Served from GitHub Pages at https://<owner>.github.io/javy/. CI sets
// `BOOK_SITE` so a fork deploys to its own owner's Pages.
const site = process.env.BOOK_SITE ?? 'https://bytecodealliance.github.io';
const base = '/javy';
const repo = 'https://github.com/bytecodealliance/javy';

// Follows the table of contents in `docs/index.md`; pages it doesn't list
// come last in their section. Pages are labeled with their own title, except
// a section's landing page, whose title would repeat the section's.
const toc = tableOfContents();
const unlisted = unlistedSlugs(toc);
const sidebar = toc.map(({ title, entries }) => {
	const section = entries[0].slug.split('/')[0];
	return {
		label: title,
		items: [
			...entries.map(({ title, slug }) => (slug.includes('/') ? slug : { label: title, slug })),
			...unlisted.filter((slug) => slug.startsWith(`${section}/`)),
		],
	};
});

export default defineConfig({
	site,
	base,
	markdown: {
		remarkPlugins: [remarkStripLeadingHeading, [remarkDocsLinks, { base, repo }], remarkMermaid],
	},
	integrations: [
		starlight({
			title: 'Javy',
			description: 'Seamless JavaScript on WebAssembly: how Javy works, and how to use and extend it.',
			social: [
				{ icon: 'github', label: 'GitHub', href: repo },
				{
					icon: 'zulip',
					label: 'Zulip',
					href: 'https://bytecodealliance.zulipchat.com/#narrow/stream/370816-javy',
				},
			],
			editLink: { baseUrl: `${repo}/edit/main/book/` },
			// The docs embed WebAssembly text; Shiki calls the grammar `wasm`.
			expressiveCode: { shiki: { langAlias: { wat: 'wasm' } } },
			customCss: [
				'@fontsource-variable/ibm-plex-sans',
				'@fontsource/ibm-plex-mono/400.css',
				'@fontsource/ibm-plex-mono/600.css',
				'./src/styles/custom.css',
			],
			lastUpdated: true,
			// Loaded as a plain script, not a `Head` override, so it can't clash
			// with themes or plugins that override components.
			head: [
				{
					tag: 'script',
					attrs: { type: 'module' },
					content: readFileSync(new URL('./src/mermaid.js', import.meta.url), 'utf8'),
				},
			],
			sidebar,
		}),
	],
});
