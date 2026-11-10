# usus

A CLI that reports AI inference usage against your allowance.

## Install

```sh
curl -fsSL https://raw.githubusercontent.com/VDuchauffour/usus/main/install.sh | bash
```

or

```sh
cargo install usus
```

or grab a [prebuilt binary](https://github.com/VDuchauffour/usus/releases).

Shell completions:

```sh
usus completion bash >~/.local/share/bash-completion/completions/usus
```

## Usage

```text
Usage: usus [PROVIDER] [ACTION]

Commands:
  opencode    Use the OpenCode GO provider
  claude      Use the Claude Code (Anthropic) provider
  completion  Generate shell completion scripts
```

Both the provider and the action are optional:

- `usus opencode` / `usus claude` — report usage (default action)
- `usus opencode login` / `usus claude login` — configure the provider
- `usus` — report via the configured default provider

See [Configuration](docs/configuration.md) for setup and provider options.

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
