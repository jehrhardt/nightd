# nightd

An agent platform you can customize to your needs.

### Prerequisites

- [mise](https://mise.jdx.dev/installing-mise.html)
- [Podman](https://podman.io/docs/installation)

### Setup

Install the development tools (Rust, uv and more) with mise:

```bash
mise trust
mise install
```

Create an OpenCode API token and store it in `.env`:

```bash
uv run scripts/refresh_opencode_keys.py
```

This runs `pnpx @opencode/console token --json` and writes the returned token to `.env`.

### Development

Run the control plane:

```bash
mise run nightd
```

Run the CLI:

```bash
mise run nightctl -- run --agent foo "Hello"
```

### Build the container images

```bash
podman build --target release-nightd --tag nightd .
podman build --target release-nightctl --tag nightctl .
```

### Run the docs

```bash
uvx zensical serve
```

## Contributing

Request features on [Discord](https://discord.gg/XzwUyUQZ9r). We don’t accept external pull requests.
