# usus

A small Rust CLI to track your AI usage. See how much of your inference allowance has been used, and by whom.

## Install

```sh
cargo install usus
```

## Usage

```text
Usage: usus <COMMAND>

Commands:
  login   Configure a provider
  report  Fetch and display current usage
```

## Providers

### OpenCode GO

```sh
usus login opencode-go
```

You will be prompted for:

- **Auth cookie** — from DevTools → Application → Cookies → `https://opencode.ai` (starts with `Fe26.2**`)
- **Workspace ID**, **Server ID**, and **Function ID** — pre-filled with sensible defaults
- **Billing cycle start day** — the day of the month your subscription renews

### Anthropic

```sh
usus login anthropic
```

You will be prompted for:

- **Admin API key** — from [console.anthropic.com/settings/admin-keys](https://console.anthropic.com/settings/admin-keys) (starts with `sk-ant-admin01-...`)
- **Monthly allowance** — defaults to `$200.00`

## Configuration

Config lives at `~/.config/usus/config.toml`. Example:

```toml
default = "anthropic"
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

- `default` — the provider used when `--provider` is omitted
- `sub_day` — day of the month the billing cycle starts (1–31)

## Development

To ensure that you follow the development workflow, please setup the pre-commit hooks:

```sh
just pre-commit-install
```

> **Note:** This requires [`uv`](https://github.com/astral-sh/uv) to be installed, as the hooks are run via `uvx pre-commit`.

Common tasks:

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
