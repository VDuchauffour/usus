# usus

A small Rust CLI to track your [OpenCode](https://opencode.ai) Go subscription usage per API key. See how much of your $60/month inference allowance has been used, and by whom.

## Install

```sh
cargo install usus
```

## Usage

```shell
Usage: usus <COMMAND>

Commands:
  login   Save your auth cookie and workspace config
  report  Fetch and display current usage
```

Run `usus login` once to paste your `auth` cookie (DevTools → Application → Cookies → `https://opencode.ai`) and set your billing-cycle start day. Re-run any time to update.

Config lives at `~/.config/usus/config.json`.

## Acknowledgments

- Initial inspiration from [jR4dh3y's opencode-go-usage](https://github.com/jR4dh3y/opencode-go-usage)
