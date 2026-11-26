# Участие в feeling-relay

[English version](CONTRIBUTING.md)

Соблюдайте [соглашение о качестве кода](CODE_QUALITY.ru.md) и пишите коммиты на английском по [Conventional Commits](COMMIT_CONVENTIONS.ru.md).

Запустите проверки из корня workspace.

```sh
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```
