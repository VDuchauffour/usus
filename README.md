# usus

A Rust CLI that reports AI inference usage.

## Install

```sh
cargo install usus
```

## Usage

```text
Usage: usus [PROVIDER] [ACTION]

Commands:
  opencode  Use the OpenCode GO provider
  anthropic    Use the Anthropic provider (personal rate limits via Claude Code OAuth)
```

Omit the provider to use the configured default. Omit the action to run `report`.

`usus anthropic` shows your personal rate-limit usage (5-hour and weekly windows)
using the Claude Code OAuth credentials at `~/.claude/.credentials.json`. No API
key required — just run `claude login` first.

`usus opencode` shows the rolling subscription usage (5-hour, weekly, and monthly
windows).

## Configuration

Config lives at `~/.config/usus/config.toml`. Example:

```toml
default_provider = "anthropic"

[providers.anthropic]

[providers."opencode"]
auth_cookie = "Fe26.2**..."
workspace_id = "wrk_01KDSXX2YK0SSF30AKBTQGWQM9"
server_id = "15702f3a12ff8bff..."
function_id = 31
```

- `default_provider` — the provider used when no provider is given on the command line
- `[providers.anthropic]` — no fields needed; reads Claude Code OAuth credentials from `~/.claude/.credentials.json` (created by `claude login`)

## Development

To ensure that you follow the development workflow, please setup the pre-commit hooks:

```sh
just pre-commit-install
```

> **Note:** This requires [`uv`](https://github.com/astral-sh/uv) to be installed, as the hooks are run via `uvx pre-commit`.

Common tasks via [`just`](https://github.com/casey/just):

```sh
just      # list all recipes
just run  # cargo run
just test # cargo test
just ci   # fmt-check + lint-strict + test
```

A [dev container](.devcontainer/devcontainer.json) is provided.

## Acknowledgments

- Initial inspiration from [jR4dh3y's opencode-go-usage](https://github.com/jR4dh3y/opencode-go-usage)
