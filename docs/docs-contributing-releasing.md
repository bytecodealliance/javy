# Crate releases

1. Merge a PR removing the `-alpha` suffix on the crate versions.
2. Change directories into the crate version you want to publish and run
   `./publish.sh`. If you're publishing `javy` and `javy-plugin-api`, publish
   the `javy` crate first.

# CLI releases

1. Merge a PR updating the root `Cargo.toml` with the new version, following
   semver.
2. Create a new release in GitHub
   [here](https://github.com/bytecodealliance/javy/releases/new). On the page
   creating the release, create a new tag and title that are set to `v`
   followed by the new version in the root `Cargo.toml`, for example `v8.0.0`.
   The `build-assets` workflow should attach artifacts to the new release.
