# src/providers/

Provider abstraction layer. Each provider owns its HTTP, parsing, and aggregation; the orchestrator (`cli/command/report.rs`) only consumes `RollingUsageView`.

## STRUCTURE

```
providers/
├── mod.rs              # Provider trait + ProviderId enum + RollingUsageView/UsageWindowView data model
├── anthropic/          # JSON-API provider (mod.rs: fetch+parse, login.rs: verify ~/.claude/.credentials.json)
└── opencode_go/        # HTML scraper (see its AGENTS.md)
```

## WHERE TO LOOK

| Concern                           | Location                                                                               | Notes                                                                                              |
| --------------------------------- | -------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------- |
| Provider trait                    | `mod.rs:29`                                                                            | 4 methods: `id`, `display_name`, `login`, `fetch_rolling_usage` (default `Ok(None)`)               |
| ProviderId enum + dispatch        | `mod.rs:45`                                                                            | `as_str()`:57, `provider()`:65 (factory → `Box<dyn Provider>`), `validate_blob()`:73, `FromStr`:87 |
| Data model (integration contract) | `mod.rs:14` (UsageWindowView), `:23` (RollingUsageView)                                | Providers produce; renderer consumes — THE seam                                                    |
| Anthropic fetch+parse             | `anthropic/mod.rs:47` (fetch), `:141` (parse_oauth_usage), `:80` (read_oauth_token)    | OAuth Bearer from ~/.claude/.credentials.json; `anthropic-beta: oauth-2025-04-20` header (:12)     |
| OpenCode fetch+parse              | `opencode_go/mod.rs:40` (fetch dispatch), `http.rs:10` (HTTP), `rolling.rs:81` (parse) | HTML scrape — see `opencode_go/AGENTS.md`                                                          |
| Per-provider Config validation    | `anthropic/mod.rs:27` (validate), `opencode_go/mod.rs:20` (validate)                   | Each re-deserializes the opaque `Value` blob into its own `Config` struct                          |

## CONVENTIONS (provider-layer specific)

- **Config crosses the trait seam as opaque `&serde_json::Value`.** The top-level `Config.providers` is `BTreeMap<String, Value>`; each provider re-deserializes into its own concrete `Config` struct *inside* `fetch_rolling_usage`. This avoids generic-over-config complexity at the trait boundary.
- **`login() -> Result<Value>` returns an opaque blob**, persisted verbatim by the orchestrator into config.toml under the provider key. Anthropic returns `json!({})` — its only "config" is the well-known credentials file path.
- **`ProviderId` is the single source of truth** for the string↔type mapping. `as_str()` = TOML key + CLI verb; `FromStr` = the reverse. `ProviderId::ALL` order (`[OpencodeGo, Anthropic]`, `mod.rs:53`) is contractual — kept stable for error-message output.
- **`fetch_rolling_usage` has a default `Ok(None)`** — providers without rolling usage need not override. The report command treats `None` as an error ("does not support rolling usage").
- **`RollingUsageView` is the integration contract.** Adding a provider = implement `Provider`, produce a `RollingUsageView`, register a `ProviderId` variant + its 4 dispatch arms. Adding a renderer = consume `RollingUsageView`. Neither side knows about the other.

## ANTI-PATTERNS (THIS PROJECT)

- **DO NOT thread raw `"anthropic"`/`"opencode"` strings** through CLI/config. Use `ProviderId`. It is the documented "type-safe replacement for the string literals that used to be threaded through" (`mod.rs:41-48`).
- **DO NOT strongly-type the top-level providers map.** It stays `BTreeMap<String, Value>`; each provider owns its `Config`.
- **DO NOT add API-key config to Anthropic.** Its `Config` is an empty struct with `deny_unknown_fields` (`anthropic/mod.rs:24`); auth is the OAuth token at `~/.claude/.credentials.json`.
- **DO NOT unify the two providers' window sets.** Anthropic emits 5-hour + Weekly (+ optional Weekly Opus/Sonnet); OpenCode emits 5-hour + Weekly + Monthly. Different semantics.

## ADDING A PROVIDER

Touch points (in order):

1. `src/providers/mod.rs:45` — add `ProviderId` variant.
2. `src/providers/mod.rs:57` — `as_str()` arm (the TOML key / CLI verb).
3. `src/providers/mod.rs:65` — `provider()` factory arm → `Box::new(...)`.
4. `src/providers/mod.rs:73` — `validate_blob()` arm → delegate to `<name>::validate`.
5. `src/providers/mod.rs:87` — `FromStr` arm.
6. `src/providers/mod.rs:53` — add to `ProviderId::ALL` (append; don't reorder existing).
7. `src/providers/<name>/mod.rs` — implement `Provider` (+ `Config` struct with `deny_unknown_fields` + `pub fn validate(&Value)`).
8. `schema/config.schema.json` — add the provider key + field schema (`additionalProperties:false`).
9. `src/cli/command/mod.rs` — optional clap subcommand; `src/main.rs` — dispatch arm.

## NOTES

- Anthropic OAuth: endpoint `https://api.anthropic.com/api/oauth/usage` (`anthropic/mod.rs:11`), beta header pinned to `oauth-2025-04-20` (:12), fallback User-Agent `claude-code/2.1.0` (:14). HTTP 401 → specific "run `claude login`" message (:63).
- OpenCode GO: `server_id`/`function_id` config fields are collected at login and schema-validated but NOT read in the current fetch path — dead fields, kept for future use.
- See `opencode_go/AGENTS.md` for the fragile HTML-scraping parser invariants.
