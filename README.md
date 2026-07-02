# usus

A Rust CLI that reports AI inference usage against your allowance.

## Install

```sh
cargo install usus
```

## Usage

```text
Usage: usus [PROVIDER] [ACTION]

Commands:
  opencode-go  Use the OpenCode GO provider
  anthropic    Use the Anthropic Admin API provider
```

Omit the provider to use the configured default. Omit the action to run `report`.

For `opencode-go`, `report` shows the rolling subscription usage (5-hour, weekly,
and monthly windows) by default. Pass `--per-keys` to show the per-key cost
breakdown instead:

```sh
usus opencode-go report --per-keys
```

## Configuration

Config lives at `~/.config/usus/config.toml`. Example:

```toml
default_provider = "anthropic"
sub_day = 5

[providers.anthropic]
admin_key = "sk-ant-admin01-..."
allowance = 200.0

[providers."opencode-go"]
auth_cookie = "Fe26.2**..."
workspace_id = "wrk_01KDSXX2YK0SSF30AKBTQGWQM9"
server_id = "15702f3a12ff8bff..."
function_id = 31
```

- `default_provider` — the provider used when no provider is given on the command line
- `sub_day` — day of the month the billing cycle starts (1–31)

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
