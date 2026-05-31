# usus

A small Rust CLI to track your [OpenCode](https://opencode.ai) Go subscription usage per API key. See how much of your $60/month inference allowance has been used, and by whom.

## Install

```sh
cargo install --path .
```

Or run from source:

```sh
cargo run --release -- <command>
```

## Usage

```sh
usus login         # save auth cookie + workspace config
usus set-sub-day   # update billing cycle start day
usus report        # fetch and display current usage
```

### First-time setup

Run `usus login` and follow the prompts. You'll need to grab your `auth` cookie from the browser:

1. Log in to your OpenCode account.
2. Open DevTools → Application → Cookies → `https://opencode.ai`.
3. Copy the `auth` cookie value

Config is stored at `~/.config/usus/config.json`.

## Acknowledgments

- Initial inspiration from [jR4dh3y's opencode-go-usage](https://github.com/jR4dh3y/opencode-go-usage)
