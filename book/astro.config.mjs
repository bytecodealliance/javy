// @ts-check
import { readFileSync } from 'node:fs';
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import { remarkDocsLinks, remarkMermaid, remarkStripLeadingHeading } from './src/remark.ts';

// Served from GitHub Pages at https://<owner>.github.io/javy/. CI sets
// `BOOK_SITE` so a fork deploys to its own owner's Pages.
const site = process.env.BOOK_SITE ?? 'https://bytecodealliance.github.io';
const base = '/javy';
const repo = 'https://github.com/bytecodealliance/javy';

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
			// Follows the table of contents in `docs/index.md`; pages it doesn't
			// list come last in their section.
			sidebar: [
				{
					label: 'Usage',
					items: [
						'using/runtime-requirements',
						'using/extending',
						'using/exports',
						'using/invoking',
						'using/js-api-support',
						'using/dynamic-linking',
						'using/nodejs',
					],
				},
				{
					label: 'Contributing',
					items: [
						{ label: 'General guidelines', slug: 'contributing' },
						'contributing/architecture',
						'contributing/building',
						'contributing/developing',
						'contributing/testing-locally',
						'contributing/releasing',
						'contributing/complex-data-types',
					],
				},
			],
		}),
	],
});
