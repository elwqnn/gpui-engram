# Changelog

All notable changes to this project will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
the project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Per-crate metadata (`description`, `repository`, `homepage`, `keywords`, `categories`) and `publish = true` on `engram`, `engram-ui`, `engram-theme`.
- `CHANGELOG.md` seeded from git history.
- GitHub Actions CI workflow (`.github/workflows/ci.yml`): `cargo check`, `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`.
- Unit tests for `ThumbMetrics` scrollbar math (`crates/engram-ui/src/components/scroll_metrics.rs`) covering degenerate viewports, short/tall content, thumb-top ↔ scroll round-trip, and the `SCROLLBAR_MIN_THUMB_RATIO` floor.
- `TextField`: word-by-word navigation (Ctrl/Alt + Left/Right, plus shift-for-select and backspace/delete-for-word variants).
- `TextField`: undo/redo (Cmd/Ctrl+Z, Cmd/Ctrl+Shift+Z, Ctrl+Y) with consecutive typing and consecutive deletions grouped into a single step.
- `TextField`: opt-in multi-line mode via `TextField::multi_line()` + `min_lines(n)` — Shift+Enter inserts a newline, Enter still submits, Up/Down navigate with preserved goal column, paste preserves newlines, height auto-grows. Hard-wrap only (no soft-wrap on width overflow).

### Changed
- `gpui` / `gpui_platform` now pinned by `rev` (`3a5dc8e`) instead of `branch = "main"`, so builds are reproducible across contributors without relying on `Cargo.lock` catching drift.
- Workspace `engram` added to `[workspace.dependencies]` so `story` no longer uses a one-off `path = ...` override.

## [0.1.0] — pre-release (on `v0.1` branch)

Component library breadth + theme refinement layer landed across the early commits. Condensed view of the component + foundation surface added so far:

### Added — components

Avatar / Facepile / Chip / CountBadge, Banner, Breadcrumb, Button family (Button, ButtonLike, ButtonLink, CopyButton, IconButton, SplitButton, ToggleButtonGroup), Callout, Checkbox, DecoratedIcon, DescriptionList, Disclosure, Divider, DropdownMenu, GradientFade, Group, Headline, HighlightedLabel, HoverCard, Icon (~140 Lucide icons), Image, Indicator, KeyBinding, KeybindingHint, Label / LabelLike, List / ListItem, Menu, Modal, Navigable, Notification, Pagination, Popover, Progress (bar + circular), Radio, Scrollbar, Sheet, Skeleton, Slider, Spinner, Stack (`h_flex` / `v_flex`), Stepper, Switch, Tab / TabBar, TextField, Tooltip, TreeViewItem, VariableList, VirtualList, Accordion.

### Added — theme & foundation

- `Theme` / `ThemeColors` / `StatusColors` / `Color` semantic enum.
- `ThemeColorsRefinement` + `ThemeContent` JSON loader + `ThemeRegistry` global.
- Hot reload (`watch_themes_dir`) and `sync_with_system_appearance`.
- Four canonical themes (engram dark / light, gruvbox dark / light).
- `StyledExt` (`h_flex`, `v_flex`, `elevation_*`, `border_*`), `ElevationIndex` (4 levels).
- `Clickable` / `Disableable` / `Toggleable` traits for naming uniformity.
- Consolidated handler aliases in `traits/handlers.rs` (`ClickHandler`, `ToggleHandler`, `StringHandler`, `DismissHandler`, `UsizeHandler`, `F32Handler`, `F64Handler`, …).
- Render smoke tests for every component (66 tests in `tests/render_smoke.rs`).
- `story` gallery binary + multi-theme showcase example.

[Unreleased]: https://github.com/Elwqnn/gpui-engram/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Elwqnn/gpui-engram/releases/tag/v0.1.0
