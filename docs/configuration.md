# Configuration

Config lives at `~/.config/usus/config.toml`:

```toml
default_provider = "anthropic"

[providers.anthropic]

[providers."opencode"]
auth_cookie = "Fe26.2**..."
workspace_id = "wrk_01KDSXX2YK0SSF30AKBTQGWQM9"
server_id = "15702f3a12ff8bff..."
function_id = 31
```

- `default_provider` — provider used when none is given on the command line. Must match a `[providers.<key>]` section.

Provider keys in the config differ from CLI commands for Claude Code:

| CLI command     | Config key  |
| --------------- | ----------- |
| `usus claude`   | `anthropic` |
| `usus opencode` | `opencode`  |

The easiest way to fill in a provider section is `usus <provider> login`, which runs the provider's login flow and persists the result for you.

## Claude Code (`usus claude`, key `anthropic`)

No fields needed. Reads Claude Code OAuth credentials from `~/.claude/.credentials.json`, created by `claude login`:

```toml
[providers.anthropic]
```

## OpenCode GO (`usus opencode`, key `opencode`)

`usus opencode login` prompts for these fields.

| Field          | Description                                   |
| -------------- | --------------------------------------------- |
| `auth_cookie`  | Session cookie from the OpenCode GO dashboard |
| `workspace_id` | Workspace to report usage for                 |
| `server_id`    | Collected at login (not currently used)       |
| `function_id`  | Collected at login (not currently used)       |

```toml
[providers."opencode"]
auth_cookie = "Fe26.2**..."
workspace_id = "wrk_01KDSXX2YK0SSF30AKBTQGWQM9"
server_id = "15702f3a12ff8bff..."
function_id = 31
```
