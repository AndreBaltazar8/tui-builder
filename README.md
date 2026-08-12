# TUI Builder

A local-first visual editor for terminal user interfaces. Designs are rendered by Ratatui itself in WebAssembly and export as standalone Rust/Ratatui applications.

Try it at **https://andrebaltazar8.github.io/tui-builder/**.

## Development

```sh
npm install
npm run dev
```

The build uses the project-local `wasm-pack` dependency:

```sh
npm run build
npm run check
```

Rust-only checks can be run with `cargo test --workspace`. A generated-project smoke test can be produced with:

```sh
cargo run -p tui-builder-core --example export_fixture -- /tmp/tui-builder-export
cargo check --manifest-path /tmp/tui-builder-export/Cargo.toml
```

## Architecture

- `crates/tui-core`: versioned project model, validation, real Ratatui buffer renderer, WASM bridge, and standalone project generator.
- `src`: React editor, local persistence, terminal cell renderer, hierarchy, widget palette, and property inspector.
- `fixtures`: portable `.tuib.json` documents used for export checks.

Projects autosave in IndexedDB and can be exchanged as `.tuib.json`. Exported Rust is a one-way scaffold with a screen module per frame and built-in `Tab`/`Shift+Tab` navigation.
