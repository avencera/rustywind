alias npm := npm-publish

default:
    just --list

fmt:
    cargo fmt

clippy:
    cargo clippy -- -D warnings
    cargo clippy -p dprint-plugin-rustywind --all-targets -- -D warnings
    cargo clippy -p dprint-plugin-rustywind --target wasm32-unknown-unknown -- -D warnings

compare-tailwind *args:
    cargo xtask compare run {{ args }}

update:
    cargo update --workspace

npm-publish:
    cargo xtask npm publish

dprint-plugin:
    bash ci/dprint_smoke.bash
