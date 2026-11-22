# nightd

An agent platform you can customize to your needs.

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Podman](https://podman.io/docs/installation)

### Development

Run the control plane:

```bash
cargo run -p nightd
```

Run the CLI:

```bash
cargo run -p nightctl
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
