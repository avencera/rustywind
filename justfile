alias npm := npm-publish

fmt:
    cargo fmt

clippy:
    cargo clippy -- -D warnings

update:
    cargo update --workspace

npm-publish:
    cargo xtask npm publish
