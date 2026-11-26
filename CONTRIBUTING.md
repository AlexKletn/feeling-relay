# Contributing to feeling-relay

[Русская версия](CONTRIBUTING.ru.md)

Follow the [code quality agreement](CODE_QUALITY.md) and write [Conventional Commits](COMMIT_CONVENTIONS.md) in English.

Run these checks from the workspace root.

```sh
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```
