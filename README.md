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
