# eTilbudsavis → Rust

A typed Rust client, command-line tool, and MCP server for **eTilbudsavis**
(`com.eTilbudsavis.eTilbudsavis`), Tjek's Danish leaflet and offer aggregator,
reverse-engineered from the Android app.

The Android app is a Capacitor shell around a React/TanStack web bundle. The
bundle's complete API surface — reads, commands, auth, and data models — was
recovered from `assets/public/assets/entry-client-*.js`; the native DEX holds
only the plugin bridge. The findings are in [`re/`](re/).

## Workspace

| Crate | Binary | Purpose |
| --- | --- | --- |
| `etilbudsavis-api` | – | Async client: catalogue, offers, stores, search, videos, auth, favourites, shopping lists |
| `etilbudsavis-cli` | `etilbudsavis` | Human-readable and `--json` CLI |
| `etilbudsavis-mcp` | `etilbudsavis-mcp` | Model Context Protocol server over stdio |

## Build

```sh
nix develop          # or: direnv allow
just build
just test
just lint
just fmt
```

The dev shell pins Rust via `rust-toolchain.toml`, includes the APK
reverse-engineering tools, and provides the cross compilers used for static
release builds.

## CLI

```sh
# Public reads, no account needed
just cli catalogue list
just cli business list --limit 10
just cli business show 365discount
just cli offer list --limit 10
just cli offer list --business 365discount --search smør
just cli store list --business 365discount
just cli search suggestions mælk
just cli search seo mælk
just cli video list --limit 5
just cli location show
just cli privacy

# Account-backed commands create an anonymous account on first use
just cli auth status
just cli lists create "Ugens indkøb"
just cli lists add 123456 "mælk" --count 2
just cli favorites add 365discount
just cli video like <video-id>

# Email one-time-code sign-in
just cli auth login you@example.com
```

Global flags include `--json`, `--token`, `--session-file`, `--base-url`,
`--api-key`, `--timeout`, `--latitude`, and `--longitude`. Run
`etilbudsavis --help` or `etilbudsavis <domain> --help` for the full surface.

The anonymous token is stored in
`~/.local/share/etilbudsavis/token.json` with mode 0600. `ETILBUDSVIS_TOKEN`
and `ETILBUDSVIS_SESSION_FILE` override it.

## MCP server

Start the read-only server:

```sh
just mcp
```

Enable the account-writing tier and, separately, the destructive tier:

```sh
just mcp --allow-writes
just mcp --allow-writes --allow-deletes
```

`just mcp-tools` prints the JSON schemas, `just mcp --check` reports the
account, and `just mcp --list-tools` includes only the enabled tier.

Example client configuration:

```json
{
  "mcpServers": {
    "etilbudsavis": {
      "command": "nix",
      "args": ["run", "github:sshine/etilbudsavis-rs#etilbudsavis-mcp"]
    }
  }
}
```

Read tools cover session status, the catalogue home screen, businesses,
offers, related offers, stores, search, videos, favourites, shopping lists,
location, and the privacy policy. Writes cover favourites, list creation and
edits, and video likes. Deletes cover list deletion, item removal, and clearing
a list.

## API in one page

- Base URL: `https://etilbudsavis.tjek.com`
- Reads: `POST /` with
  `{"data":["<base64 of [dataKey, params]>", ...]}`, answered as NDJSON.
- Commands: `POST /api/<operation>` with a JSON body.
- Headers: `x-api-key` (the public Android key), `x-token` (the account JWT),
  and `x-location` (percent-encoded JSON).
- Auth: anonymous account creation or email OTP; tokens live for about 60 days.

The full catalogue of data keys, command bodies, and response shapes is in
[`re/analysis/api.md`](re/analysis/api.md); the auth flow is in
[`re/analysis/auth.md`](re/analysis/auth.md).

## Testing

```sh
just test          # unit, wiremock, fixture, CLI, and MCP surface tests
just live-test     # opt-in tests against the real service
just showcase      # read-only CLI tour against the real service
just ci            # formatting, clippy, tests, docs, lock, packaging
```

Live tests do not need credentials: public reads use the Android API key, and
the account tests use anonymous accounts created with a throwaway token file.

## Releases

A `v*` tag runs the same CI checks, builds static
`x86_64-unknown-linux-musl` and `aarch64-unknown-linux-musl` tarballs with both
binaries, smoke-tests the x86_64 artifact, and attaches the tarballs and
SHA-256 checksums to the GitHub Release.

```sh
git tag v0.1.0
git push origin v0.1.0
```

## License

MIT OR Apache-2.0. This project is an independent client and is not affiliated
with Tjek. Use it within the service's terms and your own account's limits.
