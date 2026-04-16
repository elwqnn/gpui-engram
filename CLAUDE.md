# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`engram` is a small GPUI-based component library — a Zed-flavored UI toolkit built on `gpui` from the (unreleased) Zed source. It is a Cargo workspace with three crates that downstream apps consume through the umbrella `engram` crate (`use engram::prelude::*;`).

## Workspace layout

```
crates/
  engram/         umbrella facade — re-exports `engram_theme` and `engram_ui`
  engram-theme/   theme tokens (Color, Spacing, Radius, TextSize) + ActiveTheme global
  engram-ui/      component primitives + shared traits + embedded SVG assets
  story/          per-component story gallery binary (sidebar nav + theme switching)
reference/        excluded read-only checkout of zed-industries/zed (do NOT modify; gitignored)
```

`gpui` and `gpui_platform` are pulled directly from `zed-industries/zed` `main` (see `Cargo.toml` workspace deps), so the API surface tracks Zed's `main`. The local `reference/zed` checkout is excluded from the workspace via `exclude = ["reference"]` and is there as a read-reference only — when porting a component, look at it but don't depend on or edit it.

## Common commands

```bash
cargo build                              # build all workspace crates
cargo check                              # quick type-check
cargo clippy --all-targets               # lint
cargo test -p engram-ui                  # run engram-ui smoke tests (the only test crate)
cargo test -p engram-ui render_smoke::button_renders_every_style   # one test
cargo run -p story                       # launch the story gallery (Wayland/X11)
cargo run --example showcase -p engram   # launch the multi-theme showcase
```

The story gallery (`cargo run -p story`) is the canonical way to eyeball every component — it provides a sidebar with per-component navigation and theme switching. The showcase example shows all components in a single scrollable page across both light and dark themes.

## Architecture

### Two-step app initialization

Apps that consume engram must call **both** init functions during startup, in order:

```rust
engram_theme::init(cx);   // installs default dark Theme as a GPUI Global
engram_ui::init(cx);      // registers TextField default keybindings
```

Skipping `engram_ui::init` means `TextField` won't respond to arrow keys, copy/paste, or Enter. Skipping `engram_theme::init` means every component panics on `cx.theme()`.

To use icons, the app must also wire up the asset source: `application().with_assets(engram_ui::Assets)`. `Icon` resolves SVGs through GPUI's `AssetSource` registered on the `Application`, not through any engram-side cache.

### Theme system (engram-theme)

- `Theme { name, appearance, colors }` is stored as a `GlobalTheme(Arc<Theme>)` and accessed everywhere via the `ActiveTheme` trait (`cx.theme()`).
- Components never reach for raw `Hsla`. They take a semantic `Color` enum (`Default`, `Muted`, `Accent`, `Success`, …) and resolve it against `ThemeColors` at render time.
- `Spacing`, `Radius`, `TextSize`, `IconSize` are fixed-pixel enums. There is **no** density/dynamic-spacing dimension yet — deliberately simpler than Zed's `DynamicSpacing` proc macro. Add density only if it actually starts to matter.
- `default_dark()` / `default_light()` are hand-tuned starting themes; live theme switching is just a matter of calling `set_theme(...)` and notifying.

### Component layer (engram-ui)

Every component lives in its own file under `src/components/`. They are re-exported flat from `components.rs` and again from `src/ui.rs` plus the `prelude` module — when adding a new component, wire it through all three.

The `traits/` module deserves attention. The traits `Clickable`, `Disableable`, `Toggleable` are **not** used as generic bounds anywhere — they exist purely to enforce **naming uniformity** across unrelated structs (every component spells "click" the same way, every component spells "disabled" the same way). Don't refactor them away thinking they're dead code, and don't invent ad-hoc method names like `set_active` / `on_toggle` — route through the trait method instead.

### Handler aliases

All interactive components store callbacks as `Rc<dyn Fn(...)>` via the type aliases in `src/traits/handlers.rs`:

- `ClickHandler` — `Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>`
- `ToggleHandler` — receives the *new* `ToggleState` after the flip
- `StringHandler` — used by `TextField` for change/submit
- `DismissHandler` — for modals/popovers, no event payload (called from both Esc and backdrop-click paths)

When adding a new handler shape, **add it here** rather than declaring a local alias in your component. This was a real maintenance trap before consolidation.

`Rc` (not `Box`) is intentional: a single handler often needs to be cloned into multiple closures (e.g. an `on_click` and an `on_key_down`) within one render pass.

### ButtonLike internals

`ButtonLike` supports per-corner rounding via `ButtonLikeRounding` (used internally by `ToggleButtonGroup` for segmented controls) and optional fixed width via `.width()` / `.full_width()`. These are `pub(crate)` / `pub(super)` — consumer code should use the higher-level components that compose `ButtonLike`, not reach for rounding directly.

### Tests: render smoke tests

`crates/engram-ui/tests/render_smoke.rs` is the sole test file. Each test opens a `TestAppContext` window, builds one frame's worth of element tree inside a tiny `TestRoot` view, and asserts only that the draw pass doesn't panic. They are intentionally not pixel/snapshot tests.

Two things to know when adding a smoke test:

1. **Use `TestRoot`, not bare elements.** Interactive GPUI elements call `window.current_view()` during paint to key hitboxes — without a view on the stack they panic. The `smoke()` helper handles this.
2. **Stateful components (`TextField`, `Tooltip`) must be built inside `cx.new(...)`** — see `text_field_renders` and `tooltip_view_renders` for the pattern.

When you add a new component, add a smoke test for it in the same PR. The whole point of these tests is to catch broken `ParentElement` / `RenderOnce` wiring and bad handler-alias variance — both of which compile fine but blow up at draw time.

### Icons

`IconName` is a `strum` enum of ~140 curated icons, each variant mapping to `assets/icons/<snake_case>.svg`, embedded into the `engram-ui` binary via `rust-embed`. The SVGs are sourced from [Lucide](https://lucide.dev) (ISC License — see `crates/engram-ui/assets/icons/LICENSES`). The set is permissive-licensable and ships under engram's MIT/Apache-2.0 dual license.

To add an icon: pull the canonical SVG from `https://raw.githubusercontent.com/lucide-icons/lucide/main/icons/<lucide-name>.svg`, save it under engram's snake_case name in `crates/engram-ui/assets/icons/`, and add the variant to `IconName`. A few engram-side renames exist where the component layer already speaks differently — `Close` → Lucide `x`, `Dash` → `minus`, `Warning` → `triangle-alert`, `XCircle` → `circle-x`, `MagnifyingGlass` → `search`, `Refresh` → `refresh-cw`. Don't copy from `reference/zed/assets/icons/` — those icons are 16×16 custom Zed drawings under unclear licensing, not Lucide.

For brand/trademark icons (logos of AI providers, editors, languages, etc.) engram intentionally ships *none*. Consumer apps that genuinely integrate with a branded service should drop the brand SVG into their own asset source and use [`Icon::from_path`] — that's the layer where nominative fair use actually applies. See `crates/engram-ui/src/components/icon.rs:362`.

### TextField

`components/text_field.rs` is derived from `crates/gpui/examples/input.rs` in `zed-industries/zed` (Apache-2.0), adapted to engram's theming. The file header carries the explicit derivation notice required by Apache-2.0 §4(b). It is the only component with a custom `gpui::Element` impl, its own actions namespace (`engram_text_field`), and a process-global key-binding registration (done in `engram_ui::init`). Word-by-word navigation, undo/redo (with consecutive-keystroke grouping), and opt-in multi-line mode (`TextField::multi_line()` — hard-wrap only) are implemented. Soft-wrap on width and word-double-click selection are still TODO.

## Conventions

- **2024 edition**, resolver = "3", workspace-versioned packages (`version.workspace = true`).
- New components live under `crates/engram-ui/src/components/<name>.rs` and must be exported from three places: `components.rs`, `ui.rs`'s `pub use components::*`, and the `prelude` module's explicit list.
- Use `h_flex()` / `v_flex()` from `components::stack` instead of repeating `div().flex().flex_row().items_center()`.
- Components take semantic tokens (`Spacing::Medium`, `Color::Muted`), never raw pixels or hex colors, unless the API specifically allows escape-hatch raw values.
- Module-level rustdoc on every component file explains the *why* — keep this style when adding new ones; future readers (and future Claude sessions) rely on it.
