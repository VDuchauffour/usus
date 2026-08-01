# usus

A CLI that reports AI inference usage against your allowance.

## Install

```sh
curl -fsSL https://raw.githubusercontent.com/VDuchauffour/usus/main/install.sh | bash
```

or `cargo install usus` · [binaries](https://github.com/VDuchauffour/usus/releases)

`usus completion` prints a completion script for your current shell (detected from `$SHELL`). Pipe it into the right location for your shell, or pass a shell explicitly:

```sh
usus completion bash
usus completion zsh
usus completion fish
usus completion # autodetect from $SHELL
```

For example, with bash:

```sh
usus completion bash >~/.local/share/bash-completion/completions/usus
```

## Usage

```text
Usage: usus [PROVIDER] [ACTION]

Commands:
  opencode    Shows the rolling subscription usage
  anthropic   Shows your personal rate-limit usage
  completion  Generate shell completion scripts
```

Omit the provider to use the configured default. Omit the action to run `report`.

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
