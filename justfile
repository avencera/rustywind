alias npm := npm-publish

fmt:
    cargo fmt

clippy:
    cargo clippy -- -D warnings

npm-publish:
    cargo xtask npm publish
