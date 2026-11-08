# PRD: eTilbudsavis Android app → Rust client, CLI, and MCP server

**Status:** Complete; v0.1.0 released with static Linux binaries
**Last updated:** 2026-11-08

## 1. Summary

Reverse-engineer the Danish **eTilbudsavis** Android app
(`com.eTilbudsavis.eTilbudsavis`) into a production-quality Rust API library,
CLI, and Model Context Protocol server.

eTilbudsavis is Tjek's Danish leaflet and offer aggregator. The Android app is
a **Capacitor** shell: the native DEX is a small plugin host, and the real
application is a React/TanStack web bundle in `assets/public/`. That made the
static analysis unusually direct: the API client, request shapes, data keys,
and auth flow are all present as readable minified JavaScript.

## 2. Target

| Field | Value |
| --- | --- |
| Package | `com.eTilbudsavis.eTilbudsavis` |
| App name | eTilbudsavis |
| Version under analysis | **7.2** (versionCode 7000104) |
| minSdk / targetSdk | 24 / 36 |
| Distribution | Google Play; APKPure XAPK for 7.2 |
| Architecture | Capacitor + React/TanStack web bundle |
| Locales | Danish, Norwegian, Swedish, Finnish, English, Chinese splits |

### Acquired artifact

```text
apk/raw/com.eTilbudsavis.eTilbudsavis@7.2.xapk   16,770,233 bytes
  sha256 c3b0846f9d0d14ed1a788d9dc7756b6969a399756d9bf489c79c4c7f707bff00
apk/extracted/com.eTilbudsavis.eTilbudsavis.apk  16,577,279 bytes
  sha256 ee7e426beaabff2906ce05d41c61cdd28e8f71d362e5b5dd671cc17a2d393957
```

The XAPK contains the base APK plus `config.arm64_v8a`, `config.en`,
`config.mdpi`, and `config.zh` splits. The base APK has one main DEX
(`classes.dex`, 7.7 MiB), two small DEX files, and no native libraries; the web
bundle is under `assets/public/assets/`.

## 3. Goals

- Map the app's API, authentication, data keys, and models from the web bundle.
- Validate every read and the safe write flows against the live service using
  an anonymous account and, when the user provides one, an email OTP account.
- Implement a typed Rust library (`etilbudsavis-api`), CLI (`etilbudsavis`), and
  MCP server (`etilbudsavis-mcp`).
- Cover the client with unit tests, wiremock integration tests, fixture tests,
  and opt-in live tests.
- Publish static Linux binaries through GitHub Actions and Releases.
- Keep APKs, decompiled output, tokens, and PII out of git.

## 4. Non-goals

- Repackaging or redistributing the app.
- Bypassing authentication, paywalls, rate limits, or other access controls.
- Account deletion, user-generated video upload, ads, and surveys in v1.
- A full replacement UI or a FUSE filesystem.
- Publishing to crates.io.

## 5. Milestones

| # | Milestone | Gate | Status |
| --- | --- | --- | --- |
| M1 | Project setup, dendritic flake, PRD, APK acquisition | `nix develop` evaluates; `just fmt-check` passes | Done |
| M2 | Extract and decompile: XAPK, APK, jadx, apktool, assets | `re/analysis/` documents the bundle and manifest | Done |
| M3 | Static analysis: architecture, endpoints, auth, data keys | API/auth/model catalogue reviewed | Done |
| M4 | Live API validation: data keys, auth, search, lists | Anonymous token and write flow validated | Done |
| M5 | Rust library: client, auth, session, models, services | `cargo test` passes | Done |
| M6 | CLI: clap subcommands, human and `--json` output | `just showcase` runs against the live service | Done |
| M7 | Integration testing: wiremock, fixtures, live tests | `just test` and `just live-test` pass | Done |
| M8 | MCP server: read tools plus tiered writes | Tool surface tests pass | Done |
| M9 | Security and cleanup: PII scan, README, LICENSE, release | `just ci`; GitHub Release verified | Done |

## 6. Constraints and decisions

- **Static first, then direct API calls.** No emulator, proxy, or runtime
  instrumentation.
- **Capacitor web bundle is the specification.** The service's BFF is not the
  public Squid API; it is `https://etilbudsavis.tjek.com`, which the web bundle
  calls.
- **Nix dendritic flake.** `flake.nix` wires inputs and `import-tree ./nix`;
  each file under `nix/` is an independent flake-parts module.
- **`x-api-key` is public.** It is compiled into the Android bundle. The account
  token (`x-token`) is the credential and is stored under the XDG data
  directory with mode 0600.
- **Anonymous accounts are a supported app flow.** The app calls
  `auth-create-person` before the first shopping list or favourite. The CLI and
  MCP server do the same only when an account-backed command needs one.

## 7. API findings (summary)

- Base URL: `https://etilbudsavis.tjek.com`
- Reads: `POST /` with `{"data":["<base64 of [dataKey, params]>", ...]}` and an
  NDJSON response per requested key.
- Commands: `POST /api/<operation>` with a JSON body.
- Headers: `x-api-key`, optional `x-token`, optional `x-location`
  (percent-encoded JSON with `country`, `geohash`, `source`).
- Android API key: `590990e7ad62f206512ce390939ea3d3`.
- Auth: email OTP or anonymous person creation; tokens are JWTs valid for
  roughly 60 days.
- The full catalogue, offer, store, search, video, favourite, and shopping-list
  surfaces are documented in [`re/analysis/api.md`](analysis/api.md).

## 8. Results

- 34 read data keys and 38 command operations were enumerated.
- 27 read data keys are modelled; ads, surveys, uploads, publication-viewer
  JavaScript, and account deletion are intentionally out of scope.
- Shopping-list create/add/update/remove/delete and favourites were validated
  live against a real anonymous account.
- The CLI exposes auth, catalogue, business, offer, store, search, video,
  favourites, lists, agents, location, for-you, and privacy commands.
- The MCP server exposes read tools always and write/delete tiers behind
  `--allow-writes` and `--allow-deletes`.
