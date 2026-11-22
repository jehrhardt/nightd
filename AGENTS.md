# Guidelines for agents

This is a Rust mono-repo that uses `cargo` as a build tool. Applications live in the `apps` directory of the Cargo workspace.

## Credentials

CRITICAL: NEVER try to read or write to `.env`. ALWAYS ask the user to modify it.

## Committing

### Pre-commit Checklist

Before committing changes on code, tests or dependencies do the following tasks:

- Format code - `cargo fmt`
- Type check - `cargo check --workspace --all-targets`
- Lint - `cargo clippy --workspace --all-targets -- -D warnings`
- Run tests - `cargo test --workspace`
- Fix all errors and warnings

### Commit Messages

Use conventional commits for all commit messages.

### Developer Certificate of Origin

All commits MUST be signed off to certify the Developer Certificate of Origin. ALWAYS create commits with `git commit -s` to append a `Signed-off-by` trailer with your name and email.

```bash
git commit -s -m "feat: add something"
```

NEVER commit without a valid `Signed-off-by` line.
