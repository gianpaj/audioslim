# Agent Instructions for AudioSlim Electrobun

This folder contains the Electrobun port of AudioSlim.

## Stack

- Runtime/Main process: Bun + Electrobun
- UI: React + TypeScript + Vite
- Desktop framework: Electrobun

## Project Layout

- `src/bun/` Bun main-process logic (window, RPC handlers, conversion engine)
- `src/mainview/` React app rendered in webview
- `src/shared/` shared types and RPC schema
- `tests/e2e/` Bun e2e/integration tests
- `scripts/prepare-ffmpeg.ts` prebuild ffmpeg bundling script
- `vendors/ffmpeg/macos-arm64/ffmpeg` vendored ffmpeg binary for packaging

## Core Commands

- Install deps: `bun install`
- Dev app: `bun run dev`
- HMR dev: `bun run dev:hmr`
- Build (macos-arm64 target from config): `bun run build`
- Canary macOS arm64 build: `bun run build:canary:macos-arm64`
- Bundle ffmpeg manually: `bun run prepare:ffmpeg`
- Tests: `bun run test` or `bun run test:e2e`

## Implementation Rules

1. Keep RPC contract in `src/shared/rpc.ts` as the source of truth.
2. Long-running RPC calls must not use short timeouts (`maxRequestTime: Infinity` is currently required).
3. Audio conversion must emit per-file progress updates back to webview.
4. Before converting, detect overwrite targets and prompt user confirmation.
5. Preserve native macOS app menu roles and `Cmd+Q` quit shortcut.
6. Prefer bundled ffmpeg in packaged app; then fallback to env/system paths.

## Packaging Notes

- Current packaging flow targets **macOS arm64** first.
- ffmpeg is copied into app resources at `app/bin/ffmpeg` during Electrobun build via `build.copy`.
- `scripts.preBuild` runs `scripts/prepare-ffmpeg.ts` to ensure vendored ffmpeg exists.

## Testing Expectations

- Run `bun run test:e2e` after conversion or RPC changes.
- Run `bun run build` after build/config/runtime changes.

## Do Not

- Do not reintroduce Tauri APIs in this folder.
- Do not rely only on shell `$PATH` for ffmpeg in packaged builds.
- Do not remove overwrite confirmation behavior.
