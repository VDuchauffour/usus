# PROJECT KNOWLEDGE BASE

**Generated:** 2026-08-01
**Commit:** a012542
**Branch:** main

## OVERVIEW

Rust CLI (`usus`) that reports AI inference usage against your allowance. Two providers: Anthropic (personal rate limits via Claude Code OAuth) and OpenCode GO (rolling subscription via HTML scraping). Edition 2024, sync-only (reqwest blocking + rustls, no tokio), ratatui one-shot inline rendering.

## STRUCTURE

```
usus/
├── src/main.rs              # process entry: Cli::parse() → run(); run() is pub + testable
├── src/lib.rs               # 4-line re-export hub (cli/config/providers/ui) — exists for usus:: paths
├── src/config.rs            # config load + dual validation (JSON Schema + semantic) + provider selection
├── src/cli/                 # clap CLI + orchestration
│   ├── command/mod.rs       # two-level nested subcommands, both optional; custom clap styling
│   ├── command/report.rs    # THE 25-line orchestrator: load → pick → fetch → render (start here)
│   ├── command/login.rs     # provider-agnostic login: provider.login() → persist blob → save
│   └── render.rs            # indicatif spinner (NOT ratatui — separate from ui/render.rs)
├── src/providers/           # Provider trait + ProviderId registry + per-provider impls (see its AGENTS.md)
│   ├── mod.rs               # trait + ProviderId enum (4-hat: TOML key / CLI verb / factory / validator)
│   ├── anthropic/           # JSON-API provider, reads ~/.claude/.credentials.json, no API key
│   └── opencode_go/         # HTML scraper + hand-rolled parser (see its AGENTS.md)
├── src/ui/
│   ├── render.rs            # ratatui Viewport::Inline one-shot panel (header/separator/body)
│   └── prompt.rs            # dialoguer input primitives shared by CLI + provider login flows
└── schema/config.schema.json # draft-07 JSON Schema, embedded via include_str! — source of truth for config shape
```

## WHERE TO LOOK

| Task                              | Location                                                                | Notes                                                                    |
| --------------------------------- | ----------------------------------------------------------------------- | ------------------------------------------------------------------------ |
| Trace the full request flow       | `src/cli/command/report.rs:8`                                           | 25-line orchestrator — config→pick→fetch→render. Best start-here anchor. |
| Add/modify a provider             | `src/providers/mod.rs:29` (trait), `:45` (ProviderId), `:65` (dispatch) | See `src/providers/AGENTS.md`                                            |
| Config loading & validation       | `src/config.rs:108` (load), `:79` (schema), `:19` (semantic validate)   | Two validation layers — update both when adding fields                   |
| Provider selection logic          | `src/config.rs:43` `pick_provider_id`                                   | Precedence: CLI flag → default_provider → single configured → bail       |
| Rendering / frame layout          | `src/ui/render.rs:151` `render_rolling`                                 | header_line / separator_line / info_line / bar_line                      |
| CLI definition (clap)             | `src/cli/command/mod.rs:23`                                             | Two-level optional subcommands; `cargo_styles()`                         |
| HTTP + auth (Anthropic)           | `src/providers/anthropic/mod.rs:47`                                     | OAuth Bearer token from ~/.claude/.credentials.json                      |
| HTTP + parse (OpenCode)           | `src/providers/opencode_go/http.rs:10`, `rolling.rs:81`                 | HTML scrape, NOT an API — see opencode_go/AGENTS.md                      |
| Data model (integration contract) | `src/providers/mod.rs:14` (UsageWindowView), `:23` (RollingUsageView)   | Providers produce it; renderer consumes it — the seam                    |

## CONVENTIONS

- **anyhow everywhere**, no custom error enum, no thiserror. Idioms: `bail!("...")` for terminal user-facing errors; `anyhow::anyhow!(...)` inside `ok_or_else`; `.with_context(|| format!(...))` for I/O/parse wrapping; `.context("...")` imported as `Context as _` to avoid unused-import warnings.
- **Error messages include remediation commands** and use the real invoked program name: `env::args().next().unwrap_or_else(|| "usus".into())` (appears 3× in config.rs) — supports renamed binaries.
- **Triple-layer config validation**: (1) `#[serde(deny_unknown_fields)]` on `Config` + each provider `Config`, (2) JSON Schema (`schema/config.schema.json`, `additionalProperties:false` everywhere, embedded via `include_str!` at `config.rs:77`), (3) `Config::validate()` for cross-field semantics. Schema = shape truth; validate() = semantic truth. Keep all three in sync.
- **Two config load paths differ intentionally**: `load()` (`config.rs:108`) = full schema+semantic validation (used by report); `load_or_default()` (`config.rs:123`) = TOML deserialize only, SKIPS schema (used by login, which writes partial blobs).
- **Provider config blobs are opaque `serde_json::Value`** at the top level (`config.rs:15` `BTreeMap<String, Value>`). Each provider re-deserializes into its own `Config` struct inside `fetch_rolling_usage`. Do NOT strongly-type the top-level map.
- **`BTreeMap` not `HashMap`** for `Config.providers` — deterministic key order for stable TOML output on save. `save()` (`config.rs:134`) refuses invalid config.
- **Edition 2024 let-chains/let-else** used freely (no feature flags): `if let Some(w) = x && let Some(p) = w.y { }` (anthropic/mod.rs:177), `let Some(s) = x else { return 0 };` (anthropic/mod.rs:189). Requires Rust 1.85+.
- **Sync-only HTTP**: `reqwest::blocking` + `rustls-tls` (no native-tls, no tokio). `default-features=false` + explicit features discipline (also for chrono, jsonschema).
- **Three terminal-styling crates, scoped jobs**: `console::style` (login messages), `indicatif` (spinner during fetch), `ratatui` (final panel). Do not mix.
- **Tests are inline** `#[cfg(test)] mod tests { use super::*; ... }` — NO `tests/` dir, no integration tests, no benches. Assert style: `assert!(err.contains("..."), "got: {err}")`.
- **`.unwrap()`/`.expect()` in non-test code** only for compile-time-embedded resources or logically-guaranteed `Some`, with an explanatory `expect("...")` message (e.g. `config.rs:81,84` on `include_str!`'d schema; `config.rs:58` guarded by `len()==1`). No bare `.unwrap()` on I/O or user input.

## ANTI-PATTERNS (THIS PROJECT)

- **DO NOT use stable `cargo fmt`.** `just fmt`/`fmt-check` = `cargo +nightly fmt` — `rustfmt.toml` uses unstable `group_imports = "StdExternalCrate"`. Stable fmt will produce wrong output.
- **DO NOT delete `Cargo.lock`.** `.gitignore:155` lists it (leftover from template) but it IS committed and `just test`/`lint-strict` use `--locked`. Binary crate → lockfile must stay tracked.
- **DO NOT add an API-key config field for Anthropic.** Its `Config` is deliberately empty (`deny_unknown_fields`); auth comes from `~/.claude/.credentials.json` via `claude login`.
- **DO NOT replace `opencode_go/rolling.rs` with serde_json.** The payload is TanStack-Start serialized JS, NOT valid JSON. See `src/providers/opencode_go/AGENTS.md`.
- **DO NOT convert `ui/render.rs` to a full-screen TUI.** It uses `Viewport::Inline` (one-shot, no raw mode, no alt-screen). The `drop(terminal)` at line 43 MUST precede the `println!()` at line 44 — that ordering is load-bearing (cursor restore + shell `%` marker avoidance).
- **DO NOT thread raw `"anthropic"`/`"opencode"` strings** through CLI/config layers. Use `ProviderId` (`providers/mod.rs:45`) — the type-safe replacement. `ProviderId::ALL` order is contractual (stable for error messages).
- **DO NOT add native-tls.** `reqwest` uses `rustls-tls` with `default-features=false`.
- **DO NOT downgrade edition** or rewrite let-chains as nested `if`. Edition 2024 is required.
- **Adding a config field requires updating THREE places**: the Rust struct, `schema/config.schema.json`, and a rejection test (see `config.rs:170-216` pattern). The schema is `include_str!`'d — a schema edit needs a recompile.
- **`clippy` warnings are hard errors**: `just lint-strict` = `cargo clippy --all-targets --all-features --locked -- -D warnings`. There are NO crate-level `#![deny]` attributes — enforcement is purely external.

## UNIQUE STYLES

- **`ProviderId` wears four hats**: TOML key (`as_str()`), CLI verb (`FromStr`), factory (`provider() -> Box<dyn Provider>`), validator dispatcher (`validate_blob()`). One enum, four roles.
- **`opencode` (CLI/TOML spelling) ≠ `opencode_go` (Rust type/module).** Mapping lives only in `ProviderId::as_str()` + `FromStr`. README even mislabels it.
- **`login() -> Result<Value>` returns an opaque blob**, not a typed config. The orchestrator persists it verbatim into config.toml. Anthropic returns `json!({})` (empty) — its only "config" is a well-known file path.
- **ratatui one-shot rendering**: `Viewport::Inline(height)` draws inline and exits — no event loop. Bespoke layout: fixed `PANEL_WIDTH = 52` + manual `gap_between()` math, NOT ratatui layout constraints.
- **`cli/render.rs` (indicatif spinner) and `ui/render.rs` (ratatui panel) are different files with different jobs** — both named `render.rs`, different crates. Do not conflate.

## COMMANDS

```bash
just run [args]         # cargo run -- [args]
just test               # cargo test --locked
just check              # cargo check
just fmt                # cargo +nightly fmt            (writes; nightly REQUIRED)
just fmt-check          # cargo +nightly fmt --check
just lint-strict        # cargo clippy --all-targets --all-features --locked -- -D warnings
just ci                 # fmt-check + lint-strict + test  (read-only pre-merge gate)
just ci-fix             # fmt + lint-strict-fix + test    (auto-fixing; mutates working tree)
just build              # cargo build
just release            # cargo build --release          (lto=thin, strip=true)
just install            # cargo install --path .
just pre-commit-install # installs pre-commit + pre-push hooks (requires `uv` for uvx)
```

## NOTES

- **CI does NOT run `just test`.** GitHub Actions `ci.yml` runs only `just fmt-check` + `just lint-strict`. Tests run in CI only via the separate `coverage` job (`cargo tarpaulin`). `just ci` (local) is the stricter gate.
- **Pre-commit hooks are mandatory** (README:51): `just check` on commit, `just ci-fix` on pre-push (auto-fixes fmt+clippy then runs tests). Requires `uv` installed (`uvx pre-commit`). `just ci-fix` uses `--allow-dirty` and mutates your working tree during the hook.
- **`just bump` requires `cargo-bump`** — NOT provisioned in the devcontainer. `cargo install cargo-bump` first.
- **PR titles MUST be Conventional Commits** and the subject MUST NOT start with an uppercase letter (`subjectPattern: ^(?![A-Z]).+$`, `validateSingleCommit: true`). Enforced via `amannn/action-semantic-pull-request@v6`.
- **Branch names drive auto-labeling**: `feature/*|feat/*|fix/*|bug/*|chore/*|dependencies/*|renovate/*|update/*|bump/*|deps/*`.
- **rust-analyzer uses a separate target dir** `target/rust-analyzer` (devcontainer.json:29) to avoid lock contention with CLI builds.
- **`.omo/` is gitignored** — oh-my-openagent local state; never commit.
- **OpenCode GO `server_id`/`function_id`** are collected at login and validated by schema but NOT read in the current `http::fetch_go_page` path — dead config fields, kept for future use.
- **Renovate quirk**: `lockFileMaintenance.enabled=false` but `automerge=true` (`.github/renovate.json`). Don't "fix" without understanding intent.
