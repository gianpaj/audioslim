# Agent Instructions for AudioSlim

This document provides project-specific instructions for AI agents working on this Tauri application.

## Project Overview

AudioSlim is a desktop application built with:
- **Frontend**: React 18 + TypeScript + Vite
- **Backend**: Tauri v2 (Rust)
- **Package Manager**: pnpm
- **Plugins**: tauri-plugin-fs, tauri-plugin-shell

## Project Structure

```
audioslim/
├── src/                    # React frontend source
│   ├── App.tsx            # Main React component
│   ├── main.tsx           # React entry point
│   └── assets/            # Frontend assets
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── main.rs        # Tauri application entry
│   │   └── lib.rs         # Tauri library code
│   ├── Cargo.toml         # Rust dependencies
│   ├── tauri.conf.json    # Tauri configuration
│   └── icons/             # Application icons
├── dist/                  # Built frontend (generated)
└── package.json           # Node dependencies & scripts
```

## Development Commands

### Starting Development Server
```bash
pnpm tauri dev
```
This runs both the Vite dev server (port 1420) and the Tauri app.

### Frontend Only Development
```bash
pnpm dev
```
Runs only the Vite dev server without Tauri.

### Building for Production
```bash
pnpm tauri build
```
Compiles TypeScript, builds the frontend, and creates platform-specific binaries.

### Type Checking
```bash
pnpm build
```
Runs TypeScript compiler and builds the frontend.

## Architecture Guidelines

### Frontend (React + TypeScript)
- **Location**: `src/` directory
- **Entry Point**: `src/main.tsx`
- **Main Component**: `src/App.tsx`
- **Styling**: CSS files (e.g., `App.css`)
- **Type Definitions**: `src/vite-env.d.ts` for Vite types

### Backend (Rust + Tauri)
- **Location**: `src-tauri/src/` directory
- **Main File**: `src-tauri/src/main.rs` - Application lifecycle and setup
- **Library**: `src-tauri/src/lib.rs` - Shared library code
- **Dependencies**: Managed in `src-tauri/Cargo.toml`

### Tauri Configuration
- **Config File**: `src-tauri/tauri.conf.json`
- **App Identifier**: `com.tauri-app.app`
- **Default Window Size**: 800x600
- **Dev Server**: http://localhost:1420
- **Frontend Dist**: `../dist` (relative to src-tauri)

## Working with Tauri

### Creating Tauri Commands
1. Define Rust functions in `src-tauri/src/lib.rs` or `main.rs`
2. Annotate with `#[tauri::command]`
3. Register in the Tauri builder in `main.rs`
4. Invoke from React using `@tauri-apps/api`

Example:
```rust
// src-tauri/src/lib.rs
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
```

```typescript
// src/App.tsx
import { invoke } from '@tauri-apps/api/core';
const greeting = await invoke('greet', { name: 'World' });
```

### Using Tauri Plugins
Currently enabled plugins:
- **fs**: File system operations
- **shell**: Execute shell commands

Import and use in React:
```typescript
import { readTextFile } from '@tauri-apps/plugin-fs';
import { Command } from '@tauri-apps/plugin-shell';
```

## Best Practices

### When Making Changes

1. **Frontend Changes**:
   - Modify files in `src/`
   - Use TypeScript for type safety
   - Hot reload is enabled in dev mode
   - Test in the Tauri dev window

2. **Backend Changes**:
   - Modify files in `src-tauri/src/`
   - Follow Rust conventions and ownership rules
   - Tauri will recompile automatically in dev mode
   - Add dependencies to `Cargo.toml`

3. **Configuration Changes**:
   - Update `src-tauri/tauri.conf.json` for app settings
   - Update `package.json` for npm scripts/dependencies
   - Update `Cargo.toml` for Rust dependencies

### Code Organization

- **Keep Tauri commands focused**: Each command should do one thing well
- **Handle errors properly**: Use `Result<T, E>` in Rust, try/catch in TypeScript
- **Type everything**: Leverage TypeScript and Rust's type systems
- **Separate concerns**: Keep business logic separate from UI components

### Security Considerations

- **CSP**: Currently set to `null` in config - consider hardening for production
- **Validate inputs**: Always validate data passed from frontend to Tauri commands
- **Permissions**: Review plugin permissions before adding new capabilities
- **Allowlist**: Consider using Tauri's allowlist for commands in production

## Common Workflows

### Adding a New Feature

1. Plan the feature (frontend/backend split)
2. If backend changes needed:
   - Create Tauri command in `src-tauri/src/lib.rs`
   - Register in `main.rs`
   - Update `Cargo.toml` if new deps needed
3. If frontend changes needed:
   - Create/modify React components in `src/`
   - Add types in TypeScript
   - Invoke Tauri commands as needed
4. Test with `pnpm tauri dev`
5. Build with `pnpm tauri build` to verify production build

### Adding Dependencies

**Frontend (npm)**:
```bash
pnpm add <package-name>
pnpm add -D <package-name>  # dev dependency
```

**Backend (Rust)**:
Edit `src-tauri/Cargo.toml` or use:
```bash
cd src-tauri
cargo add <crate-name>
```

### Debugging

- **Frontend**: Use browser DevTools (enabled in Tauri dev window)
- **Backend**: Use `println!()` or `dbg!()` macros, output appears in terminal
- **Tauri API**: Check console for API errors

### Building for Release

1. Update version in both `package.json` and `src-tauri/Cargo.toml`
2. Update version in `src-tauri/tauri.conf.json`
3. Run `pnpm tauri build`
4. Find binaries in `src-tauri/target/release/bundle/`

## Testing

- **Frontend**: Add test framework (e.g., Vitest, Jest)
- **Backend**: Use `cargo test` in `src-tauri/`
- **Integration**: Consider Tauri's WebDriver for E2E tests

## Important Notes

- **Never commit** `target/`, `dist/`, `node_modules/`
- **Package manager**: This project uses `pnpm`, not npm or yarn
- **Rust edition**: 2021 (check `Cargo.toml`)
- **Tauri version**: v2 (breaking changes from v1)
- **Window size**: Default 800x600, configurable in `tauri.conf.json`

## Resources

- [Tauri Documentation](https://tauri.app/v2/guides/)
- [Tauri API Reference](https://tauri.app/v2/reference/javascript/api/)
- [React Documentation](https://react.dev/)
- [Vite Documentation](https://vitejs.dev/)
- [Rust Documentation](https://doc.rust-lang.org/)

## Getting Help

When encountering issues:
1. Check Tauri documentation for v2-specific guidance
2. Review Cargo.toml and package.json for version conflicts
3. Clear build cache: `rm -rf src-tauri/target dist`
4. Reinstall dependencies: `pnpm install && cd src-tauri && cargo clean`

---

**Last Updated**: 2026-02-07
