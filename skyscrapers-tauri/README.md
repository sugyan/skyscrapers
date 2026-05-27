# skyscrapers-tauri

Native desktop / mobile app for the Skyscrapers puzzle, built on
[Tauri v2](https://v2.tauri.app/). Wraps the existing
[`skyscrapers-player`](../skyscrapers-player) React component and serves
puzzle generation / hint lookups from the in-process Rust engine via
Tauri commands — no WebAssembly at runtime.

## Layout

- `src/` — Vite + React + TypeScript frontend. Reuses `skyscrapers-player`
  for the UI and provides `TauriEngine` (in `src/engine/`) as a
  `SkyscrapersEngine` implementation that calls Rust via
  `@tauri-apps/api` `invoke()`.
- `src-tauri/` — Tauri host crate. `src/commands.rs` exposes the two
  `#[tauri::command]` handlers (`generate_puzzle`, `next_hint`) backed
  by `skyscrapers-generator` / `skyscrapers-solver`.

## Develop

```sh
cd skyscrapers-tauri
npm install
npm run tauri dev      # vite dev server + native window with hot reload
```

The first run can take a while as Cargo compiles Tauri's transitive deps.

## Build

```sh
npm run tauri build    # produces a release binary + OS-specific bundle
```

On macOS the artefacts land in
`../target/release/bundle/macos/skyscrapers-tauri.app` (and
`../target/release/bundle/dmg/` if `bundle_dmg.sh` succeeds).

## CI / workspace note

This crate is intentionally excluded from the workspace `default-members`
in the root `Cargo.toml` because the Tauri build pulls in OS-level deps
(WebView2 / WKWebView / webkit2gtk + glib) that the existing Rust CI on
`ubuntu-latest` does not provide. Plain `cargo build` / `cargo test`
from the repo root therefore skip this crate. Build it via
`npm run tauri build` (or `cargo build -p skyscrapers-tauri` after
installing the platform deps).

Tauri-specific CI lives in three workflows:

- `.github/workflows/tauri-check.yml` runs on every PR that touches
  files reachable from the Tauri bundle. A 3-OS matrix
  (macOS / Ubuntu 22.04 / Windows) executes `tauri-action --no-bundle`
  as a build smoke test — Rust + frontend compile, no installer assembly.
- `.github/workflows/tagpr.yml` watches `main` and uses
  [Songmu/tagpr](https://github.com/Songmu/tagpr) to maintain a single
  "release skyscrapers-tauri vX.Y.Z" PR that bumps
  `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`
  in lockstep (see `.tagpr` at the repo root).
- `.github/workflows/tauri-release.yml` is triggered by the
  `skyscrapers-tauri-v*` tag that tagpr cuts when its PR is merged.
  It builds `.app` / `.dmg` (arm64 + x86_64), `.AppImage` / `.deb`, and
  `.msi` in parallel and uploads them to a draft GitHub Release. The
  macOS bundle is signed with the Developer ID certificate and notarised
  if the `APPLE_*` repository secrets are present.

## Release

1. Land regular feature PRs on `main`. Add a `tagpr:minor` or
   `tagpr:major` label if you want the next release to bump beyond the
   default `patch`.
2. tagpr opens / refreshes a release PR on `main` after each merge.
   Merge it when you want to cut a release.
3. The resulting `skyscrapers-tauri-vX.Y.Z` tag triggers
   `tauri-release.yml`. Wait for the matrix to finish, then publish the
   draft Release.
4. The Windows installer is currently unsigned — users will see a
   SmartScreen warning ("More info" → "Run anyway").

## Mobile

iOS and Android support are not initialised yet. They are added on top
of this same project with:

```sh
rustup target add aarch64-apple-ios x86_64-apple-ios aarch64-apple-ios-sim
npm run tauri ios init

rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
npm run tauri android init
```

iOS requires macOS + Xcode (and the Apple Developer Program for
on-device / store distribution). Android requires Android Studio + JDK +
NDK with `JAVA_HOME` / `ANDROID_HOME` / `NDK_HOME` set.
