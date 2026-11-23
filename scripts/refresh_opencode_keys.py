#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.13"
# dependencies = []
# ///

"""Create an OpenCode Console token and store it in the project's .env file."""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from datetime import UTC, datetime, timedelta
from pathlib import Path

ENV_KEY = "OPENCODE_API_KEY"
TOKEN_COMMAND = ("pnpx", "--yes", "@opencode/console", "token", "--json")
REPO_ROOT = Path(__file__).resolve().parent.parent


@dataclass(frozen=True, slots=True)
class Token:
    access_token: str
    expires_at: datetime | None


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=f"Create an OpenCode Console token and set {ENV_KEY} in .env.",
    )
    parser.add_argument(
        "--env-file",
        type=Path,
        default=REPO_ROOT / ".env",
        help="dotenv file to update (default: the project's .env)",
    )
    return parser.parse_args()


def fetch_token() -> Token:
    if shutil.which("pnpx") is None:
        raise SystemExit("pnpx not found. Install the project tools with `mise install`.")

    completed = subprocess.run(
        TOKEN_COMMAND,
        capture_output=True,
        text=True,
        stdin=subprocess.DEVNULL,
        check=False,
    )
    if completed.returncode != 0:
        detail = completed.stderr.strip() or completed.stdout.strip()
        raise SystemExit(
            f"`{' '.join(TOKEN_COMMAND)}` failed"
            f"{f':\n{detail}' if detail else f' with exit status {completed.returncode}'}"
        )

    try:
        payload = json.loads(completed.stdout)
    except json.JSONDecodeError as error:
        raise SystemExit(f"Token command returned invalid JSON: {error}") from error
    if not isinstance(payload, dict):
        raise SystemExit("Token command returned no JSON object.")

    token = payload.get("access_token")
    if not isinstance(token, str) or not token.strip():
        raise SystemExit("Token command returned no non-empty `access_token`.")
    return Token(access_token=token.strip(), expires_at=parse_expiry(payload))


def parse_expiry(payload: dict[str, object]) -> datetime | None:
    expires_in = payload.get("expires_in")
    if isinstance(expires_in, bool) or not isinstance(expires_in, (int, float)):
        return None
    return datetime.now(UTC) + timedelta(seconds=float(expires_in))


def format_value(value: str) -> str:
    if re.fullmatch(r"[A-Za-z0-9_./:@%+-]*", value):
        return value
    return json.dumps(value)


def write_atomic(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    mode = path.stat().st_mode & 0o777 if path.exists() else 0o600
    descriptor, temporary = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    try:
        with os.fdopen(descriptor, "w", encoding="utf-8", newline="\n") as handle:
            handle.write(content)
        os.chmod(temporary, mode)
        os.replace(temporary, path)
    except BaseException:
        Path(temporary).unlink(missing_ok=True)
        raise


def update_env_file(path: Path, key: str, value: str) -> bool:
    """Set `key` in the dotenv file, returning True when an existing entry was replaced."""
    path = path.expanduser()
    line_pattern = re.compile(
        rf"^(?P<indent>[ \t]*)(?P<export>export[ \t]+)?{re.escape(key)}[ \t]*=.*$"
    )
    entry = f"{key}={format_value(value)}"

    lines = path.read_text(encoding="utf-8").splitlines() if path.exists() else []
    replaced = False
    updated: list[str] = []
    for line in lines:
        match = line_pattern.match(line)
        if match is None:
            updated.append(line)
            continue
        prefix = f"{match.group('indent')}{match.group('export') or ''}"
        updated.append(f"{prefix}{entry}")
        replaced = True

    if not replaced:
        updated.append(entry)

    write_atomic(path, "\n".join(updated) + "\n")
    return replaced


def main() -> int:
    args = parse_args()
    token = fetch_token()
    replaced = update_env_file(args.env_file, ENV_KEY, token.access_token)
    action = "Updated" if replaced else "Added"
    print(f"{action} {ENV_KEY} in {args.env_file}")
    if token.expires_at is None:
        print("Token expiry not reported by OpenCode Console.")
    else:
        print(f"Token expires at {token.expires_at.isoformat(timespec='seconds')}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
