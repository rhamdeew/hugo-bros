# Repository Guidelines

## Project Structure & Module Organization
- `src/` holds the SvelteKit frontend (`routes/` for pages, `lib/` for shared components).
- `src-tauri/` contains the Rust backend and Tauri config (`src-tauri/src/`, `tauri.conf.json`).
- `static/` is for static assets served by SvelteKit.
- `example_blog/` is a sample Hexo blog fixture for local testing.
- Build output lives in `build/`, `dist/`, and `.svelte-kit/` (generated).

## Build, Test, and Development Commands
- `npm run dev` or `make dev`: run the Vite dev server (frontend only).
- `npm run dev:app` or `make dev-app`: run the full app (Vite + Tauri).
- `npm run build` or `make build`: build the frontend.
- `npm run build:tauri` or `make build-tauri`: build frontend + Tauri app.
- `npm run check` or `make check`: run Svelte/TypeScript type checks.
- `npm run lint` or `make lint`: run ESLint.
- `npm run format` or `make format`: format with Prettier.
- `make cargo-check`, `make cargo-clippy`, `make cargo-test`: Rust checks and tests.

## Coding Style & Naming Conventions
- Frontend formatting is enforced by Prettier (`tabWidth: 2`, `singleQuote: true`, `printWidth: 100`).
- ESLint runs for `.ts` and `.svelte`; prefix unused args/vars with `_` to avoid warnings.
- Keep file and component names consistent with existing SvelteKit patterns under `src/`.

## Testing Guidelines
- Rust tests run via `cargo test` or `make test`.
- There is no JavaScript test runner configured yet; use `npm run check` for TS/Svelte correctness.

## Commit & Pull Request Guidelines
- Commit messages are short, imperative, and capitalized (e.g., `Add Hexo integration`).
- PRs should include a concise description, testing notes, and UI screenshots when behavior changes.

## Configuration & Assets
- Tauri settings live in `src-tauri/tauri.conf.json`.
- Tailwind/PostCSS settings are in `tailwind.config.js` and `postcss.config.js`.
