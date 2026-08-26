alias npm := npm-publish

default:
    just --list

fmt:
    cargo fmt

clippy:
    cargo clippy -- -D warnings

compare-tailwind *args:
    cargo xtask compare run {{ args }}

update:
    cargo update --workspace

npm-publish:
    cargo xtask npm publish
