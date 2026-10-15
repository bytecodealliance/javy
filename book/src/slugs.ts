/**
 * Maps a file in `docs/` to its page slug in the book, e.g.
 * `docs-using-exports.md` -> `using/exports` and
 * `docs-contributing.md` -> `contributing`.
 */
export function docSlug(file: string): string {
	const name = file.replace(/^.*\//, '').replace(/\.md$/, '').replace(/^docs-/, '');
	const section = name.match(/^(using|contributing)-(.+)$/);
	return section ? `${section[1]}/${section[2]}` : name;
}
