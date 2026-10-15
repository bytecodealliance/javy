import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { defineCollection } from 'astro:content';
import { glob, type Loader } from 'astro/loaders';
import { docsSchema } from '@astrojs/starlight/schema';
import { docSlug } from './slugs';

// The book reads the Markdown in the repository's top-level `docs/`
// directory, so the docs stay readable on GitHub and there is a single source
// of truth. `docs/index.md` is a GitHub-only table of contents; the book uses
// its own home page and sidebar instead.
const files = glob({
	pattern: ['docs-*.md'],
	base: '../docs',
	generateId: ({ entry }) => docSlug(entry),
});

// Starlight requires a `title` in each page's frontmatter. The docs don't
// carry frontmatter, so default the title to the page's leading heading
// (which `remarkStripLeadingHeading` removes from the rendered body).
function leadingHeading(filePath: string): string | undefined {
	const match = readFileSync(filePath, 'utf8').match(/^\s*#{1,6}\s+(.+?)\s*$/m);
	return match?.[1];
}

// Starlight's `lastUpdated` only reads git history for its default
// `src/content/docs/` directory, so look up the date of the last commit that
// touched each file in `docs/` ourselves. Files with no commits yet get no date.
function lastCommitDate(filePath: string): Date | undefined {
	const log = spawnSync('git', ['log', '-1', '--format=%cI', '--', filePath], { encoding: 'utf8' });
	const date = log.stdout?.trim();
	return date ? new Date(date) : undefined;
}

const docs: Loader = {
	name: 'javy-docs-loader',
	load: (context) =>
		files.load({
			...context,
			parseData: (props) => {
				const lastUpdated = props.filePath && lastCommitDate(props.filePath);
				return context.parseData({
					...props,
					data: {
						title: props.filePath && leadingHeading(props.filePath),
						...(lastUpdated && { lastUpdated }),
						...props.data,
					},
				});
			},
		}),
};

export const collections = {
	docs: defineCollection({ loader: docs, schema: docsSchema() }),
};
