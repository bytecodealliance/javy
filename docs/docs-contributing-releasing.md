# CLI releases

1. Update the root `Cargo.toml` with the new version, following semver.
2. Create a tag for the new version like `v0.2.0`.

```
git tag v0.2.0
git push origin --tags
```

3. Create a new release from the new tag in github
   [here](https://github.com/bytecodealliance/javy/releases/new).
4. A GitHub Action will trigger for `publish.yml` when a release is published
   ([i.e. it doesn't run on
   drafts](https://docs.github.com/en/actions/using-workflows/events-that-trigger-workflows#:~:text=created%2C%20edited%2C%20or%20deleted%20activity%20types%20for%20draft%20releases)),
   creating the artifacts for downloading.


