# src/providers/opencode_go/

OpenCode GO provider — an HTML scraper, NOT an API client. Fetches the workspace go page and parses a TanStack-Start serialized JS payload by hand. The parsing is intentionally fragile and locked by tests; read this before touching `rolling.rs`.

## STRUCTURE

```
opencode_go/
├── mod.rs       # Provider impl + Config struct (4 fields, deny_unknown_fields) + validate()
├── http.rs      # fetch_go_page(): GET the workspace HTML page with spoofed browser UA + cookie
├── rolling.rs   # parse_rolling_usage(): hand-rolled string scanner over TanStack-Start JS payload
└── login.rs     # interactive login: prompts for auth_cookie/workspace_id/server_id/function_id
```

## WHERE TO LOOK

| Concern                        | Location                               | Notes                                                                                                  |
| ------------------------------ | -------------------------------------- | ------------------------------------------------------------------------------------------------------ |
| Provider impl / fetch dispatch | `mod.rs:40`                            | delegates to http.rs + rolling.rs                                                                      |
| Config struct + validate       | `mod.rs:11` (struct), `:20` (validate) | fields: auth_cookie, workspace_id, server_id, function_id                                              |
| HTTP fetch                     | `http.rs:10` `fetch_go_page`           | GETs the opencode.ai workspace go page; spoofed Chrome UA + `Cookie: oc_locale=en; auth={auth_cookie}` |
| Payload parser                 | `rolling.rs:81` `parse_rolling_usage`  | hand-rolled string scanner; `extract_window`:43, `number_after`:33, `looks_signed_out`:26              |
| Login                          | `login.rs:8`                           | prompts; validates auth_cookie starts with the iron-seal prefix `Fe26.2**` (:24)                       |

## CONVENTIONS (opencode_go-specific)

- **HTML scrape, not JSON API.** `http.rs` spoofs a Chrome User-Agent and sends `Cookie: oc_locale=en; auth={auth_cookie}`. The response is HTML embedding a TanStack-Start serialized JS payload — NOT valid JSON, NOT a REST response.
- **`rolling.rs` is a hand-rolled string scanner.** `extract_window` (rolling.rs:43) finds a key, bounds to the next `}` brace, then `number_after` (rolling.rs:33) pulls digits for `usagePercent` / `resetInSec`. The brace bounding is load-bearing — see anti-patterns.
- **`looks_signed_out` (rolling.rs:26) sniffs three sentinel strings** in the HTML: `auth/authorize`, `not associated with an account`, `actor of type "public"`. If parse fails AND any sentinel is present, the error tells the user to re-login (rolling.rs:74) instead of a generic parse failure.
- **`renews` is derived from the monthly window** `resetInSec` via `Local::now() + Duration::seconds(...)` then formatted `dd Mon YYYY` (rolling.rs:104). Anthropic derives it differently — do not unify.
- **`auth_cookie` must start with `Fe26.2**`** (the iron-seal cookie prefix). Validated at login (`login.rs:24`) and documented in the JSON schema description.

## ANTI-PATTERNS (THIS PROJECT)

- **DO NOT replace `rolling.rs` with `serde_json::from_str`.** The payload is TanStack-Start serialized JS, NOT valid JSON. The module header comment (rolling.rs:1-9) explains why. Test `skips_bare_monthly_usage_cost_field` (rolling.rs:144) locks the behavior.
- **DO NOT remove the `{...}` brace bounding in `extract_window`.** It exists to skip the bare `monthlyUsage:<bigint>` cost field, which a naive `number_after` would otherwise mis-read as the usage percent. The brace bound mirrors the upstream regex lookahead.
- **DO NOT remove or reorder the `looks_signed_out` sentinels** without updating the `errors_when_signed_out` test (rolling.rs).
- **DO NOT add `server_id`/`function_id` usage to `http.rs`.** They are collected at login and schema-validated but intentionally dead in the current fetch path — kept for future use.
- **DO NOT change the spoofed User-Agent or cookie header format** without verifying the page still serves the TanStack payload.

## NOTES

- `parse()` entry (rolling.rs:64) wraps `parse_rolling_usage` and is what `fetch_rolling_usage` (mod.rs:40) calls.
- Test fixtures use a `const PAGE` built from `concat!` of raw `r#"..."#` segments (rolling.rs:117) — no external fixture files.
- The parser lineage comes from `jR4dh3y/opencode-go-usage` (see root README acknowledgments).
- See parent `src/providers/AGENTS.md` for the Provider trait, ProviderId dispatch, and the `RollingUsageView` integration contract.
