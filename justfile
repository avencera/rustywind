alias pn := publish-npm

fmt:
    cargo fmt

clippy:
    cargo clippy -- -D warnings

publish-npm:
    cargo xtask npm publish
