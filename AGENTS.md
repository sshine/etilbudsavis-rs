# etilbudsavis-rs — agent guide

`etilbudsavis-rs` reverse-engineers the Danish **eTilbudsavis** Android app
(`com.eTilbudsavis.eTilbudsavis`) into a Rust API library, CLI, and MCP server.
Read [`re/prd.md`](re/prd.md) before starting work; it is the source of truth
for scope, milestones, and current status.

## Non-negotiable rules

1. **One logical change per commit.** Keep the history readable.
2. **All reverse-engineering artifacts go under `re/`.** Decompiled output,
   extracted splits, and probes are generated and gitignored; findings, scripts,
   and the PRD are tracked.
3. **Never commit the APK, extracted splits, decompiled output, network
   captures, or secrets.** They are covered by `.gitignore`; do not defeat it
   with `git add -f`.
4. **Tokens, PII, and credentials go in `secrets/`** (gitignored). Captured
   API responses can contain personal data: anonymise before quoting them in
   tracked notes.
5. **Use the flake.** Run commands as `nix develop --command <tool>` or enter
   the shell with `direnv allow`. Do not assume host tools exist.
6. **Do not bypass access controls.** Authenticate as yourself, use your own
   account, and stay within the app's intended API.
7. **Run the regression gate before committing:** `just fmt-check`, then
   `just ci` once the crates exist.

## Repository layout

| Path | Purpose |
| --- | --- |
| `apk/` | Downloaded XAPK and extracted splits (gitignored) |
| `re/` | PRD, scripts, analysis notes, generated decompilation (gitignored) |
| `nix/` | Dendritic flake-parts modules, auto-imported by `import-tree` |
| `crates/` | Rust workspace members |
| `secrets/` | Tokens, captured responses, anything with PII (gitignored) |
| `scripts/` | Repository-local helper scripts |

## Commands

```sh
nix develop              # enter the devshell (or `direnv allow`)
just fmt                 # format Nix, Rust, Markdown, TOML, YAML, shell
just fmt-check           # CI formatting check
just test                # workspace tests
just lint                # workspace clippy
just cli --help          # CLI
just mcp-tools           # print the MCP tool schemas
just showcase            # read-only CLI smoke test against the live service
just live-test           # real-service API tests, one at a time
just apk-info            # package/version/SDKs from the extracted base APK
```

## API notes

- Base: `https://etilbudsavis.tjek.com`
- Reads: `POST /` with `{"data":["<base64 of [key, params]>", ...]}` and NDJSON
  responses.
- Commands: `POST /api/<operation>` with a JSON body.
- Headers: `x-api-key` (the Android key from the APK), `x-token` (account
  token), `x-location` (URL-encoded JSON).
- Auth: email OTP (`auth-otp-initialize`/`auth-otp-finalize`) or anonymous
  account creation (`auth-create-person`).

## Phase loop

The workflow is the nine-phase loop in the `android-app-reverser` skill:

1. Project setup (complete)
2. Extract and decompile (complete)
3. Static analysis (complete)
4. Live API validation (complete)
5. Rust library (complete)
6. CLI (complete)
7. Integration testing (complete)
8. Advanced features (none requested; MCP server is included)
9. Security and cleanup (complete; v0.1.0 released)
