# Release

Set the release version once and reuse it:

```sh
version=0.25.2
tag=v$version
```

## Checklist

1. Update versions and changelogs.
   - Bump `rustywind-cli/Cargo.toml` and `rustywind-dprint/Cargo.toml` to the same version.
   - Bump `rustywind-core/Cargo.toml` and `rustywind-vite/Cargo.toml` only when those crates change.
   - Keep path dependency versions in sync.
   - In `CHANGELOG.md`, move the `Unreleased` notes under `## [$version] - YYYY-MM-DD`. The release workflow refuses a tag without a dated, non-empty entry, and uses that entry as the GitHub release notes.
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
just dprint-plugin
```

3. Commit the release prep.

```sh
git status --short
git add CHANGELOG.md Cargo.lock rustywind-cli/Cargo.toml rustywind-dprint/Cargo.toml rustywind-core/Cargo.toml rustywind-core/CHANGELOG.md rustywind-vite/Cargo.toml npm/packages
git commit -m "Release $tag"
git push
```

Stage only the files that actually changed.

4. Tag and push the release.

```sh
git tag "$tag"
git push origin "$tag"
```

Pushing the tag starts `.github/workflows/release.yml`, which:

- checks that `rustywind`, `dprint-plugin-rustywind` and every npm package carry the tag's version, and that `CHANGELOG.md` has a dated entry for it
- builds the binaries and the dprint plugin into a draft GitHub release, then publishes the release once every file is attached
- publishes `rustywind_core`, `rustywind_vite` and `rustywind` to crates.io, skipping versions that are already there
- downloads the release binaries into the npm packages and publishes them to npm, skipping versions that are already there
- builds and pushes the Docker images

crates.io and npm trust the workflow through OIDC (trusted publishing), so no registry token is stored. See [One-time setup](#one-time-setup).

5. Watch the release workflow.

```sh
gh run list --workflow Release --limit 3
gh run watch
```

If a registry publish fails, use "Re-run failed jobs". Every publish skips what is already uploaded, so a rerun finishes a partial release.

To publish crates.io and npm for a tag whose GitHub release is already published, without rebuilding it, run the workflow by hand:

```sh
gh workflow run release.yml -f tag="$tag"
```

6. Update the custom Homebrew tap.

Use the tap repo script after the GitHub release is published:

```sh
cd ../homebrew-taps
./update rustywind
git status --short
git push
```

7. Smoke test installs.

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

## One-time setup

The registries trust `release.yml` by file name and environment, so renaming the workflow or an environment means updating every trusted publisher.

### GitHub environments

The `crates-io` and `npm` environments hold the publish jobs. They accept `v*` tags, for tag pushes, and `master`, for manual runs:

```sh
for env in crates-io npm; do
  gh api -X PUT "repos/avencera/rustywind/environments/$env" --input - <<< \
    '{"deployment_branch_policy": {"protected_branches": false, "custom_branch_policies": true}}'
  gh api -X POST "repos/avencera/rustywind/environments/$env/deployment-branch-policies" -f name='v*' -f type=tag
  gh api -X POST "repos/avencera/rustywind/environments/$env/deployment-branch-policies" -f name=master -f type=branch
done
```

### crates.io

For each of `rustywind`, `rustywind_core` and `rustywind_vite`, open the crate's settings on crates.io (`https://crates.io/crates/<crate>/settings`) and add a GitHub trusted publisher:

- Repository owner: `avencera`
- Repository name: `rustywind`
- Workflow filename: `release.yml`
- Environment: `crates-io`

Trusted publishing can't create a crate, so publish a new crate's first version by hand with `cargo publish`.

### npm

Trusted publishing needs npm 11.5.1 or newer. Log in with `npm login`, then trust the workflow for every package:

```sh
for pkg in rustywind rustywind-darwin-arm64 rustywind-darwin-x64 rustywind-linux-arm-gnueabihf \
  rustywind-linux-arm64-gnu rustywind-linux-arm64-musl rustywind-linux-x64-musl \
  rustywind-win32-ia32-msvc rustywind-win32-x64-msvc; do
  npx -y npm@11 trust github "$pkg" --repo avencera/rustywind --file release.yml --env npm --allow-publish --yes
done
```

Once a release has published through the workflow, set each package's publishing access to require two-factor authentication and disallow tokens.

## Publishing by hand

If the workflow can't publish, publish crates in dependency order, skipping unchanged crates:

```sh
cargo publish -p rustywind_core
cargo publish -p rustywind_vite
cargo publish -p rustywind
```

Then publish npm after the GitHub release has every asset:

```sh
cargo xtask npm prepare-binaries "$tag"
cargo xtask npm publish
```

`xtask` verifies package versions, prepared binary hashes, and the local host binary version before publishing, and skips packages whose version npm already has.
