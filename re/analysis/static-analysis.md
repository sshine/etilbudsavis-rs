# Static analysis notes

## Artifact

- `com.eTilbudsavis.eTilbudsavis` 7.2, versionCode 7000104, minSdk 24,
  targetSdk 36, compileSdk 36.
- XAPK splits: base, `config.arm64_v8a`, `config.en`, `config.mdpi`,
  `config.zh`.
- Base APK: one main DEX (7.7 MiB) plus two small DEX files; no `.so` files and
  no Xamarin/Flutter/React Native payload.
- `assets/capacitor.config.json` names the app and the Capgo updater endpoints
  (`https://etilbudsavis.tjek.com/updates/check` and `/updates/stats`).
- `assets/capacitor.plugins.json` lists the native plugins: App, Browser,
  Clipboard, Device, Geolocation, Haptics, Keyboard, Network, Preferences, Push
  Notifications, Share, Splash Screen, App Review, Badge, and Capacitor
  Updater.

## Web bundle

- `assets/public/assets/entry-client-C4ir-6Gi.js` is 1.3 MiB and contains the
  application. `server.browser-rbM4s4iK.js` is the SSR renderer; the app is
  otherwise client-rendered.
- The bundle identifies itself as a TanStack Router/React application with
  Tailwind CSS. It uses a small internal data-cache framework that serializes
  `[dataKey, params]` pairs, base64-encodes them, and batches them to `POST /`.
- Platform detection picks the API key: browser
  `152000596c6e45d9983eab0c14afebea`, iOS
  `25f11322a52e7abbcf9c445222e9380e`, Android
  `590990e7ad62f206512ce390939ea3d3`.
- The same function returns `apiUrl: https://squid-api.tjek.com`, but the
  client-side paths use the BFF origin `https://etilbudsavis.tjek.com`.
- Native account state is read from Capacitor Preferences key `tjek-user: { email, token }`; location from `eta-location`; client id from
  `eta-client-id`.
- The app sends `x-api-key`, `x-token`, and `x-location`. The location header
  is `encodeURIComponent(JSON.stringify({country, geohash, source}))`.

## Native shell

jadx produced 6,290 Java files and apktool produced the decoded manifest and
resources. The DEX is dominated by Capacitor plugins and AndroidX; there is no
business logic worth reversing beyond the plugin surface. The only interesting
native behaviour is the Capacitor bridge, which forwards web `fetch` calls and
preferences.

## Why static analysis was enough

Unlike apps that hide their API in native code, this app's API surface is
entirely in JavaScript. The bundle provided:

- the exact endpoint names and request bodies,
- the data keys and parameter shapes,
- the auth flow and storage keys,
- the API key and header construction,
- model field names through property access.

Live probes then filled in response shapes and the few defaults the bundle did
not spell out (for example, which optional fields a 422 rejects).
