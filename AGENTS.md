# Guidelines for agents

This is a Rust and TypeScript mono-repo. Rust uses `cargo`, TypeScript uses `pnpm`, and both toolchains are managed by `mise`. Applications live in `apps`.

## Credentials

CRITICAL: NEVER try to read or write to `.env`. ALWAYS ask the user to modify it.

## Committing

### Pre-commit Checklist

Before committing changes on code, tests or dependencies do the following tasks:

- Format TypeScript code - `pnpm format`
- Run all checks - `mise run ci`
- Fix all errors and warnings

### Documentation

Before committing changes on the documentation or the page overrides, verify the documentation builds without errors:

```bash
uvx zensical build --clean
```

Then verify the result visually with screenshots of the built site:

```bash
python3 -m http.server 8123 -d site &
uvx playwright screenshot --full-page --viewport-size "1280,900" http://127.0.0.1:8123/ /tmp/shot.png
```

Inspect the screenshots and only commit once the rendered page matches the intent. Clean up afterwards (stop the server, remove temporary files).

### Commit Messages

Use conventional commits for all commit messages.
