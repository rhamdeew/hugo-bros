# Hex Tool

Hex Tool is a desktop GUI editor for Hexo blogs built with Tauri, SvelteKit, and TypeScript. It provides a fast, native-feeling workflow for editing posts and managing content while keeping the project structure on disk.

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).

## Getting Started

Install dependencies:

```sh
npm install
```

Run the desktop app in development mode:

```sh
npm run dev:app
```

Run frontend-only development:

```sh
npm run dev
```

## Key Commands

- `npm run build`: build the frontend.
- `npm run build:tauri`: build the full Tauri app.
- `npm run check`: run Svelte/TypeScript checks.
- `npm run lint`: run ESLint.
- `npm run format`: format code with Prettier.
- `make cargo-check`: check the Rust backend in `src-tauri/`.

## Project Layout

- `src/`: SvelteKit frontend (`routes/` and `lib/`).
- `src-tauri/`: Rust backend and Tauri configuration.
- `static/`: static assets served by SvelteKit.
- `example_blog/`: sample Hexo blog for local testing.
