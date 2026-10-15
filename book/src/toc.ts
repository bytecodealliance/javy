import { readdirSync, readFileSync } from 'node:fs';
import { docSlug } from './slugs';

/** A page listed in the table of contents. */
export interface Entry {
	title: string;
	slug: string;
}

/** A section of the table of contents, e.g. "Usage". */
export interface Section {
	title: string;
	/** The prose between the section heading and its list, if any. */
	intro?: string;
	entries: Entry[];
}

const docs = '../docs';

/**
 * Reads the table of contents in `docs/index.md`, the single source for the
 * order of the docs in the book's home page and sidebar.
 */
export function tableOfContents(): Section[] {
	const sections: Section[] = [];
	let intro: string[] = [];
	for (const line of readFileSync(`${docs}/index.md`, 'utf8').split('\n')) {
		const heading = line.match(/^##\s+(.+?)\s*$/);
		const entry = line.match(/^\s*[*-]\s+\[(.+?)\]\(\.\/(docs-[^)]+\.md)\)/);
		const section = sections.at(-1);
		if (heading) {
			sections.push({ title: heading[1], entries: [] });
			intro = [];
		} else if (entry && section) {
			section.entries.push({ title: entry[1], slug: docSlug(entry[2]) });
		} else if (section && section.entries.length === 0 && line.trim()) {
			intro.push(line.trim());
			section.intro = intro.join(' ');
		}
	}
	return sections;
}

/**
 * Slugs of the pages in `docs/` that the table of contents doesn't list, by
 * file name.
 */
export function unlistedSlugs(sections: Section[]): string[] {
	const listed = new Set(sections.flatMap((s) => s.entries.map((e) => e.slug)));
	return readdirSync(docs)
		.filter((file) => /^docs-.+\.md$/.test(file))
		.sort()
		.map(docSlug)
		.filter((slug) => !listed.has(slug));
}
