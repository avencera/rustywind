# Release

Set the release version once and reuse it:

```sh
version=0.25.2
tag=v$version
```

## Checklist

1. Update versions and changelogs.
   - Bump `rustywind-cli/Cargo.toml`.
   - Bump `rustywind-core/Cargo.toml` and `rustywind-vite/Cargo.toml` only when those crates change.
   - Keep path dependency versions in sync.
   - Update `CHANGELOG.md`.
   - Update `rustywind-core/CHANGELOG.md` when `rustywind_core` user-visible behavior changes.
   - Update npm package versions:

```sh
cargo xtask npm update-version "$version"
```

2. Verify locally.

```sh
just fmt
just clippy
cargo test -p xtask
```

3. Commit the release prep.

```sh
git status --short
git add CHANGELOG.md Cargo.lock rustywind-cli/Cargo.toml rustywind-core/Cargo.toml rustywind-core/CHANGELOG.md rustywind-vite/Cargo.toml npm/packages
git commit -m "Release $tag"
git push
```

Stage only the files that actually changed.

4. Publish crates.io packages.

Publish dependency crates first, then the CLI:

```sh
cargo publish -p rustywind_core
cargo publish -p rustywind_vite
cargo publish -p rustywind
```

Skip unchanged crates.

5. Tag and push the release.

```sh
git tag "$tag"
git push origin "$tag"
```

Pushing the tag starts `.github/workflows/mean_bean_deploy.yml`, which creates the GitHub release, uploads binaries, builds Docker images, and publishes npm packages.

6. Watch the GitHub release workflow.

```sh
gh run list --workflow "Mean Bean Deploy" --limit 3
gh run watch
```

Wait for all release assets to upload before publishing npm manually. The npm release uses those assets.

7. Publish npm if CI did not.

The preferred path is GitHub Actions with npm Trusted Publishing. If publishing locally, make sure you are logged in and then run:

```sh
cargo xtask npm bump "$tag"
```

or:

```sh
cargo xtask npm prepare-binaries "$tag"
cargo xtask npm publish
```

`xtask` verifies package versions, prepared binary hashes, and the local host binary version before publishing.

8. Update the custom Homebrew tap.

Use the tap repo script after the GitHub release assets exist:

```sh
cd ../homebrew-taps
./update rustywind
git status --short
git push
```

9. Smoke test installs.

```sh
cargo uninstall rustywind || true
npm uninstall -g rustywind || true
brew uninstall rustywind || true

cargo install rustywind --version "$version" --force
which rustywind
rustywind -V

cargo uninstall rustywind || true
npm install -g rustywind@"$version"
which rustywind
rustywind -V

npm uninstall -g rustywind || true
brew update
brew install avencera/tap/rustywind
which rustywind
rustywind -V
```

Each command should report the release version.
