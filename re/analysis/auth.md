# Authentication

The app has two account modes, and both produce the same kind of token.

## Anonymous account

`ensureAuth` in the bundle calls:

```text
POST /api/auth-create-person
{ "apiKey": "590990e7ad62f206512ce390939ea3d3" }
```

The response is a `person` object and a `token`. The token is a JWT with
`uid`, `cid`, `aid`, `pro`, `iat`, and `exp`; observed validity is 60 days.
It is stored by the web bundle in Capacitor Preferences under `tjek-user` as
`{ email, token }`, and the native context copies `token` into `x-token` for
every request.

The CLI and MCP server store it in
`~/.local/share/etilbudsavis/token.json` (mode 0600), or in
`--session-file`/`ETILBUDSVIS_SESSION_FILE`.

## Email one-time code

```text
POST /api/auth-otp-initialize
{ "apiKey": "...", "email": "you@example.com" }
  -> { "otpId": "..." }

POST /api/auth-otp-finalize
{ "apiKey": "...", "otpId": "...", "otp": "123456" }
  -> { "token": "...", "email": "you@example.com", "person": ...? }
```

The app then stores `{ email, token }` and refetches the `auth` data key.
`auth login <email>` in the CLI prompts for the code on stdin; `auth verify <otp-id> <code>` does the two steps separately for scripted use.

Error messages observed for bad codes include `Too many login attempts` and
`Model not found`; the app maps both to a "request a new code" message.

## Sign-out

The native app clears the stored user object. The browser build calls
`auth-terminate`; the CLI does the same best-effort and then removes the local
token file, so a network failure cannot leave the user signed in locally.

## Profile updates

`auth-update-person` accepts `countryCode`, `privacyPolicyConsented`, and
`enabledNotifications`. The app derives the country from the location and sends
it once per session. The CLI exposes `auth consent`, `auth notifications`, and
`auth country`.

## What is not implemented

- Account deletion (`auth-delete-person-initialize`/`finalize`) is a
  destructive, email-verified flow and is outside v1.
- Push registration and the app-install upsert are platform concerns.
- There is no password login; the app has no password form.
