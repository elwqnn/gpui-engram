//! Render smoke tests for every public engram-ui component.
//!
//! These tests are intentionally dumb: they open a test window with a root
//! view that builds the component under test, and verify the draw pass
//! doesn't panic. They don't assert on pixel output or on specific element
//! trees — GPUI's layout engine is the unit under inspection, not ours.
//!
//! Why bother? Two reasons:
//!
//! 1. **Catching regressions on the element-tree builders.** Swapping a
//!    `.child()` for an `AnyElement` can silently break `ParentElement` /
//!    `RenderOnce` wiring. A draw is the cheapest way to catch that at CI.
//! 2. **Exercising the handler-consolidation refactor.** Many components
//!    store `Rc<dyn Fn(...)>` handlers — a botched type alias would fail
//!    to compile, but a botched *variance* or a missing trait bound can
//!    still slip past the compiler and blow up only at draw time.
//!
//! ## Why a `TestRoot` view?
//!
//! `TestAppContext::draw()` draws *bare* elements — but interactive GPUI
//! elements (Button, List, Tab, anything with `.id(...)`) call
//! `window.current_view()` during paint to key their hitboxes to a view id.
//! That panics when there is no view on the stack. So each test is rendered
//! inside a tiny [`TestRoot`] entity built by `add_window_view`, which
//! gives the paint path a real view id to work with.

use std::cell::RefCell;

use engram_theme::{self, Radius, Spacing};
use engram_ui::components::{
    Avatar, AvatarSize, Banner, Button, ButtonCommon, ButtonStyle, Checkbox, CheckboxSize, Chip,
    ChipStyle, CountBadge, Disclosure, Divider, Facepile, Headline, HeadlineSize, Icon,
    IconButton, IconName, IconSize, IconSource, Image, Indicator, KeyBinding, Label, LabelCommon,
    LabelSize, List, ListItem, ListItemSpacing, Menu, Modal, Notification, Popover, Scrollbar,
    Severity, Switch, Tab, TabBar, TextField, TintColor, Tooltip, anchored_popover, h_flex,
    menu, modal_overlay, v_flex,
};
use engram_ui::traits::{Clickable, Disableable, StyledExt, ToggleState, Toggleable};
use gpui::{
    AnyElement, App, Bounds, Context, Corner, CursorStyle, IntoElement, ParentElement, Render,
    ScrollHandle, Styled, TestAppContext, Window, div, point, prelude::*, px, size,
};

type BuildFn = Box<dyn FnMut(&mut Window, &mut Context<TestRoot>) -> AnyElement>;

/// Minimal view used as the root of a smoke-test window. Holds a closure
/// that builds one frame's worth of element tree. The closure is wrapped in
/// a `RefCell` so the `&mut self` render signature doesn't conflict with
/// the closure's own captured `FnMut` state.
struct TestRoot {
    build: RefCell<BuildFn>,
}

impl Render for TestRoot {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut build = self.build.borrow_mut();
        div().size_full().p(Spacing::Medium.pixels()).child(build(window, cx))
    }
}

/// Open a test window, install the theme, and run a single draw cycle with
/// the caller-supplied build closure as the root element.
fn smoke<F>(cx: &mut TestAppContext, build: F)
where
    F: FnMut(&mut Window, &mut Context<TestRoot>) -> AnyElement + 'static,
{
    let (_view, _vtx) = cx.add_window_view(|_window, cx| {
        engram_theme::init(cx);
        engram_ui::init(cx);
        TestRoot {
            build: RefCell::new(Box::new(build)),
        }
    });
    // `add_window_view` drives a render via `run_until_parked`; the fact
    // that we returned from it without panicking is the assertion.
}

#[gpui::test]
fn label_renders(cx: &mut TestAppContext) {
    smoke(cx, |_, _| Label::new("hello").into_any_element());
}

#[gpui::test]
fn button_renders_every_style(cx: &mut TestAppContext) {
    smoke(cx, |_, _| {
        h_flex()
            .gap(Spacing::Small.pixels())
            .child(Button::new("btn-filled", "Filled"))
            .child(Button::new("btn-subtle", "Subtle").style(ButtonStyle::Subtle))
            .child(Button::new("btn-outlined", "Outlined").style(ButtonStyle::Outlined))
            .child(
                Button::new("btn-tinted-accent", "Accent")
                    .style(ButtonStyle::Tinted(TintColor::Accent)),
            )
            .child(
                Button::new("btn-tinted-error", "Delete")
                    .style(ButtonStyle::Tinted(TintColor::Error)),
            )
            .child(Button::new("btn-disabled", "Disabled").disabled(true))
            .child(Button::new("btn-icon", "With icon").icon(IconName::Check))
            .into_any_element()
    });
}

#[gpui::test]
fn icon_button_renders(cx: &mut TestAppContext) {
    smoke(cx, |_, _| {
        h_flex()
            .gap(Spacing::Small.pixels())
            .child(IconButton::new("icon-btn", IconName::Plus))
            .child(IconButton::new("icon-btn-disabled", IconName::Close).disabled(true))
            .child(
                IconButton::new("icon-btn-cursor", IconName::Check)
                    .cursor_style(CursorStyle::PointingHand),
            )
            .into_any_element()
    });
}

#[gpui::test]
fn icon_renders_every_size(cx: &mut TestAppContext) {
    smoke(cx, |_, _| {
        h_flex()
            .gap(Spacing::Small.pixels())
            .child(Icon::new(IconName::Check).size(IconSize::XSmall))
            .child(Icon::new(IconName::Check).size(IconSize::Small))
            .child(Icon::new(IconName::Check).size(IconSize::Medium))
            .child(Icon::new(IconName::Check).size(IconSize::Large))
            .child(Icon::new(IconName::Check).size(IconSize::Custom(gpui::rems(1.5))))
            .into_any_element()
    });
}

#[gpui::test]
fn icon_source_variants_render(cx: &mut TestAppContext) {
    // Exercises the three `IconSource` branches so any future wiring
    // regression in the render match arms blows up at draw time. The
    // external paths are mock strings — gpui's AssetSource resolution just
    // no-ops on lookup failure, which is fine for a draw smoke test.
    smoke(cx, |_, _| {
        h_flex()
            .gap(Spacing::Small.pixels())
            .child(Icon::new(IconName::Check))
            .child(Icon::new(IconSource::ExternalSvg("file:///mock/icon.svg".into())))
            .child(Icon::from_path("brand/engram.svg"))
            .into_any_element()
    });
}

#[gpui::test]
fn checkbox_renders_tri_state(cx: &mut TestAppContext) {
    smoke(cx, |_, _| {
        v_flex()
            .gap(Spacing::Small.pixels())
            .child(Checkbox::new("cb-unchecked", ToggleState::Unselected))
            .child(Checkbox::new("cb-checked", ToggleState::Selected))
            .child(
                Checkbox::new("cb-indeterminate", ToggleState::Indeterminate)
                    .size(CheckboxSize::Large),
            )
            .child(Checkbox::new("cb-labeled", ToggleState::Selected).label("Remember me"))
            .into_any_element()
    });
}

#[gpui::test]
fn switch_renders(cx: &mut TestAppContext) {
    smoke(cx, |_, _| {
        v_flex()
            .gap(Spacing::Small.pixels())
            .child(Switch::new("sw-off", ToggleState::Unselected))
            .child(Switch::new("sw-on", ToggleState::Selected).label("Notifications"))
            .child(Switch::new("sw-disabled", ToggleState::Selected).disabled(true))
            .into_any_element()
    });
}

#[gpui::test]
fn disclosure_renders(cx: &mut TestAppContext) {
    smoke(cx, |_, _| {
        v_flex()
            .gap(Spacing::Small.pixels())
            .child(Disclosure::new("d-open", true))
            .child(Disclosure::new("d-closed", false))
            .into_any_element()
    });
}

#[gpui::test]
fn divider_renders_both_orientations(cx: &mut TestAppContext) {
    smoke(cx, |_, _| {
        v_flex()
            .gap(Spacing::Small.pixels())
            .child(Divider::horizontal())
            .child(
                h_flex()
                    .h(px(40.0))
                    .child(Divider::vertical())
                    .child(Label::new("after vertical divider")),
            )
            .into_any_element()
    });
}

#[gpui::test]
fn indicator_renders(cx: &mut TestAppContext) {
    smoke(cx, |_, _| {
        h_flex()
            .gap(Spacing::Small.pixels())
            .child(Indicator::dot())
            .child(Indicator::dot().color(engram_theme::Color::Success))
            .child(Indicator::bar())
            .child(Indicator::icon(Icon::new(IconName::Check)))
            .into_any_element()
    });
}

#[gpui::test]
fn keybinding_renders(cx: &mut TestAppContext) {
    smoke(cx, |_, _| {
        h_flex()
            .gap(Spacing::Small.pixels())
            .child(KeyBinding::new(["Ctrl", "S"]))
            .child(KeyBinding::new(["Ctrl", "Shift", "P"]))
            .into_any_element()
    });
}

#[gpui::test]
fn list_renders_with_items(cx: &mut TestAppContext) {
    smoke(cx, |_, _| {
        List::new()
            .header("Navigation")
            .child(
                ListItem::new("li-file")
                    .start_slot(Icon::new(IconName::File))
                    .child(Label::new("File")),
            )
            .child(
                ListItem::new("li-folder")
                    .start_slot(Icon::new(IconName::Folder))
                    .child(Label::new("Folder"))
                    .toggle_state(ToggleState::Selected),
            )
            .child(
                ListItem::new("li-disabled")
                    .start_slot(Icon::new(IconName::Close))
                    .child(Label::new("Disabled"))
                    .disabled(true),
            )
            .into_any_element()
    });
}

#[gpui::test]
fn empty_list_renders(cx: &mut TestAppContext) {
    smoke(cx, |_, _| {
        List::new().empty_message("Nothing here yet").into_any_element()
    });
}

#[gpui::test]
fn list_item_phase8_fields_render(cx: &mut TestAppContext) {
    // Phase 8 acceptance: a single ListItem must accept indent_level=2,
    // dense spacing, and inset=true at once. Also exercises outlined,
    // hover-only end slot, and the new on_hover / on_secondary_mouse_down
    // handlers — all of which are wired through the trait/handler aliases,
    // so a botched type alias would only blow up at draw time.
    smoke(cx, |_, _| {
        v_flex()
            .gap(Spacing::Small.pixels())
            .child(
                ListItem::new("li-tree-leaf")
                    .indent_level(2)
                    .spacing(ListItemSpacing::Dense)
                    .inset(true)
                    .start_slot(Icon::new(IconName::File))
                    .child(Label::new("nested.rs"))
                    .end_slot(Icon::new(IconName::ChevronRight))
                    .on_hover(|_, _, _| {})
                    .on_secondary_mouse_down(|_, _, _| {}),
            )
            .child(
                ListItem::new("li-outlined-card")
                    .outlined()
                    .rounded(false)
                    .spacing(ListItemSpacing::ExtraDense)
                    .child(Label::new("card-style row")),
            )
            .child(
                ListItem::new("li-hover-end-slot")
                    .show_end_slot_on_hover()
                    .child(Label::new("hover for actions"))
                    .end_slot(Icon::new(IconName::Trash)),
            )
            .into_any_element()
    });
}

#[gpui::test]
fn tab_bar_renders(cx: &mut TestAppContext) {
    smoke(cx, |_, _| {
        TabBar::new()
            .child(Tab::new("tab-1", "First").toggle_state(true))
            .child(Tab::new("tab-2", "Second"))
            .child(Tab::new("tab-3", "Closable").on_close(|_, _, _| {}))
            .into_any_element()
    });
}

#[gpui::test]
fn avatar_and_facepile_render(cx: &mut TestAppContext) {
    smoke(cx, |_, _| {
        v_flex()
            .gap(Spacing::Small.pixels())
            .child(
                h_flex()
                    .gap(Spacing::Small.pixels())
                    .child(Avatar::new("Alice").size(AvatarSize::Small))
                    .child(Avatar::new("Bob").size(AvatarSize::Medium))
                    .child(Avatar::new("Carol").size(AvatarSize::Large)),
            )
            .child(
                Facepile::new()
                    .push(Avatar::new("Alice"))
                    .push(Avatar::new("Bob"))
                    .push(Avatar::new("Carol"))
                    .push(Avatar::new("Dan")),
            )
            .into_any_element()
    });
}

#[gpui::test]
fn image_renders(cx: &mut TestAppContext) {
    // Image paints its container even if the source can't be resolved —
    // all we're proving here is that the builder and the child wiring
    // don't panic.
    smoke(cx, |_, _| {
        h_flex()
            .gap(Spacing::Small.pixels())
            .child(Image::new("https://example.invalid/a.png").size(px(32.0)))
            .child(
                Image::new("https://example.invalid/b.png")
                    .size(px(48.0))
                    .rounded(Radius::Medium),
            )
            .child(
                Image::new("https://example.invalid/c.png")
                    .size(px(48.0))
                    .rounded_full(),
            )
            .into_any_element()
    });
}

#[gpui::test]
fn avatar_with_image_renders(cx: &mut TestAppContext) {
    smoke(cx, |_, _| {
        h_flex()
            .gap(Spacing::Small.pixels())
            .child(Avatar::new("Alice").image("https://example.invalid/alice.png"))
            .child(
                Avatar::new("Bob")
                    .size(AvatarSize::Large)
                    .image("https://example.invalid/bob.png"),
            )
            .into_any_element()
    });
}

#[gpui::test]
fn chip_and_count_badge_render(cx: &mut TestAppContext) {
    smoke(cx, |_, _| {
        h_flex()
            .gap(Spacing::Small.pixels())
            .child(Chip::new("default"))
            .child(Chip::new("accent").style(ChipStyle::Accent))
            .child(CountBadge::new(3))
            .child(CountBadge::new(100))
            .into_any_element()
    });
}

#[gpui::test]
fn banner_every_severity_renders(cx: &mut TestAppContext) {
    smoke(cx, |_, _| {
        v_flex()
            .gap(Spacing::Small.pixels())
            .child(Banner::new(Severity::Info, "Info"))
            .child(Banner::new(Severity::Success, "Success"))
            .child(Banner::new(Severity::Warning, "Warning"))
            .child(
                Banner::new(Severity::Error, "Error")
                    .description("Something went wrong.")
                    .on_dismiss(|_, _, _| {}),
            )
            .into_any_element()
    });
}

#[gpui::test]
fn notification_renders(cx: &mut TestAppContext) {
    smoke(cx, |_, _| {
        Notification::new(Severity::Success, "Saved")
            .description("Changes were saved automatically.")
            .into_any_element()
    });
}

#[gpui::test]
fn popover_container_renders(cx: &mut TestAppContext) {
    smoke(cx, |_, _| {
        Popover::new()
            .min_width(px(200.0))
            .child(Label::new("Popover contents"))
            .into_any_element()
    });
}

#[gpui::test]
fn anchored_popover_renders_with_focus_handle(cx: &mut TestAppContext) {
    smoke(cx, |_, cx| {
        let focus_handle = cx.focus_handle();
        anchored_popover(
            focus_handle,
            Corner::TopLeft,
            Bounds::new(point(px(100.0), px(100.0)), size(px(120.0), px(32.0))),
            Popover::new().child(Label::new("Anchored content")),
            |_, _| {},
        )
        .into_any_element()
    });
}

#[gpui::test]
fn menu_renders_all_item_kinds(cx: &mut TestAppContext) {
    // Menu is a stateful entity (Phase 9), so it must be constructed inside
    // `cx.new` — matching the TextField / Tooltip pattern.
    smoke(cx, |_, cx| {
        let menu = cx.new(|cx| {
            Menu::new(cx)
                .header("File")
                .entry_with_icon("m-new", IconName::Plus, "New", |_, _, _| {})
                .keybinding_entry("m-save", "Save", ["Ctrl", "S"], |_, _, _| {})
                .separator()
                .disabled_entry("m-dis", "Disabled")
        });
        menu.into_any_element()
    });
}

#[gpui::test]
fn menu_select_next_advances_cursor(cx: &mut TestAppContext) {
    // Phase 9 acceptance: dispatching `SelectNext` on a fresh menu moves the
    // keyboard cursor to the first selectable entry. A header counts as
    // non-selectable, so the first landing index is 1 (the "new" entry),
    // not 0 (the "File" header).
    let (_root, vtx) = cx.add_window_view(|_window, cx| {
        engram_theme::init(cx);
        engram_ui::init(cx);
        TestRoot {
            build: RefCell::new(Box::new(|_, _| div().into_any_element())),
        }
    });

    let menu = vtx.new(|cx| {
        Menu::new(cx)
            .header("File")
            .entry("m-new", "New", |_, _, _| {})
            .entry("m-save", "Save", |_, _, _| {})
    });

    menu.update_in(vtx, |m, window, cx| {
        m.select_next(&menu::SelectNext, window, cx);
    });
    vtx.update(|_, cx| {
        assert_eq!(menu.read(cx).selected_index(), Some(1));
    });

    menu.update_in(vtx, |m, window, cx| {
        m.select_next(&menu::SelectNext, window, cx);
    });
    vtx.update(|_, cx| {
        assert_eq!(menu.read(cx).selected_index(), Some(2));
    });
}

#[gpui::test]
fn modal_renders(cx: &mut TestAppContext) {
    smoke(cx, |_, _| {
        Modal::new()
            .title("Delete?")
            .child(Label::new("This cannot be undone."))
            .footer(
                h_flex()
                    .gap(Spacing::Small.pixels())
                    .child(Button::new("cancel", "Cancel"))
                    .child(
                        Button::new("delete", "Delete")
                            .style(ButtonStyle::Tinted(TintColor::Error)),
                    ),
            )
            .into_any_element()
    });
}

#[gpui::test]
fn modal_overlay_renders(cx: &mut TestAppContext) {
    smoke(cx, |_, cx| {
        let focus_handle = cx.focus_handle();
        modal_overlay(
            focus_handle,
            Modal::new()
                .title("Confirm")
                .child(Label::new("Are you sure?"))
                .footer(h_flex().child(Button::new("ok", "OK"))),
            |_, _| {},
        )
        .into_any_element()
    });
}

#[gpui::test]
fn text_field_renders(cx: &mut TestAppContext) {
    smoke(cx, |_, cx| {
        // TextField is stateful, so it has to be constructed inside `cx.new`.
        let field = cx.new(|cx| TextField::with_value(cx, "initial").placeholder("type…"));
        field.into_any_element()
    });
}

#[gpui::test]
fn tooltip_view_renders(cx: &mut TestAppContext) {
    smoke(cx, |_, cx| {
        let tooltip = cx.new(|_| Tooltip::new("Tooltip text").meta("Secondary info"));
        tooltip.into_any_element()
    });
}

#[gpui::test]
fn labels_take_every_size_and_color(cx: &mut TestAppContext) {
    smoke(cx, |_, _| {
        v_flex()
            .gap(Spacing::XXSmall.pixels())
            .child(Label::new("XSmall").size(LabelSize::XSmall))
            .child(Label::new("Small").size(LabelSize::Small))
            .child(Label::new("Default").size(LabelSize::Default))
            .child(Label::new("Large").size(LabelSize::Large))
            .child(Label::new("muted").color(engram_theme::Color::Muted))
            .child(Label::new("accent").color(engram_theme::Color::Accent))
            .into_any_element()
    });
}

#[gpui::test]
fn label_modifiers_compose(cx: &mut TestAppContext) {
    // Phase 5 acceptance: italic + underline + truncate must compose
    // through `LabelCommon` without any "the chain returned a different
    // type" surprises. We also exercise alpha + strikethrough + single_line
    // so a future refactor that drops a builder method gets caught here.
    smoke(cx, |_, _| {
        v_flex()
            .gap(Spacing::XXSmall.pixels())
            .child(
                Label::new("very long label that should be truncated to an ellipsis")
                    .italic()
                    .underline()
                    .truncate(),
            )
            .child(Label::new("Discounted").strikethrough().alpha(0.5))
            .child(Label::new("Line A\nLine B\nLine C").single_line())
            .into_any_element()
    });
}

#[gpui::test]
fn headline_renders_every_size(cx: &mut TestAppContext) {
    smoke(cx, |_, _| {
        v_flex()
            .gap(Spacing::XSmall.pixels())
            .child(Headline::new("XSmall").size(HeadlineSize::XSmall))
            .child(Headline::new("Small").size(HeadlineSize::Small))
            .child(Headline::new("Medium"))
            .child(Headline::new("Large").size(HeadlineSize::Large))
            .child(Headline::new("XLarge").size(HeadlineSize::XLarge))
            .into_any_element()
    });
}

#[gpui::test]
fn scrollbar_renders_both_axes(cx: &mut TestAppContext) {
    // The scrollbar reads geometry from a ScrollHandle, which is empty on
    // the first frame — so the thumb renders at zero size, but nothing
    // should panic.
    smoke(cx, |_, _| {
        let handle = ScrollHandle::new();
        v_flex()
            .gap(Spacing::Small.pixels())
            .child(
                h_flex()
                    .h(px(120.0))
                    .child(
                        div()
                            .id("scroll-body")
                            .w(px(200.0))
                            .h_full()
                            .overflow_y_scroll()
                            .track_scroll(&handle)
                            .child(div().h(px(600.0)).w_full().bg(gpui::white())),
                    )
                    .child(Scrollbar::vertical(handle.clone())),
            )
            .child(Scrollbar::horizontal(ScrollHandle::new()))
            .into_any_element()
    });
}

#[gpui::test]
fn radius_tokens_apply(cx: &mut TestAppContext) {
    // Exercise the Radius token path via a styled container — this catches
    // any Radius variant that was added but not wired through Pixels().
    smoke(cx, |_, _| {
        h_flex()
            .gap(Spacing::Small.pixels())
            .child(
                div()
                    .w(px(32.0))
                    .h(px(32.0))
                    .bg(gpui::white())
                    .rounded(Radius::None.pixels()),
            )
            .child(
                div()
                    .w(px(32.0))
                    .h(px(32.0))
                    .bg(gpui::white())
                    .rounded(Radius::Small.pixels()),
            )
            .child(
                div()
                    .w(px(32.0))
                    .h(px(32.0))
                    .bg(gpui::white())
                    .rounded(Radius::Medium.pixels()),
            )
            .child(
                div()
                    .w(px(32.0))
                    .h(px(32.0))
                    .bg(gpui::white())
                    .rounded(Radius::Large.pixels()),
            )
            .child(
                div()
                    .w(px(32.0))
                    .h(px(32.0))
                    .bg(gpui::white())
                    .rounded(Radius::Full.pixels()),
            )
            .into_any_element()
    });
}

#[gpui::test]
fn styled_ext_elevation_renders(cx: &mut TestAppContext) {
    // Proves the StyledExt blanket impl is wired: a bare Div can chain
    // h_flex() and elevation_2(cx) and make it through a draw cycle.
    smoke(cx, |_, cx| {
        div()
            .h_flex()
            .elevation_2(cx)
            .child(Label::new("elevated"))
            .into_any_element()
    });
}

#[gpui::test]
fn json_theme_registry_pipeline_renders(cx: &mut TestAppContext) {
    // Proves the JSON → ThemeRegistry → activate_theme → component-render
    // pipeline end-to-end. Loads gruvbox_dark.json from the embedded asset
    // bundle, parses it via `Theme::from_json_bytes`, inserts it into the
    // global registry, switches the active theme to it, and then draws a
    // Banner per status hue plus an accent Indicator. Catches drift between
    // `default_dark`'s schema and the canonical JSON shape (status keys
    // missing, refinement layer broken, etc.) — anything in that chain
    // would either fail to parse or panic during the draw.
    use gpui::AssetSource;
    smoke(cx, |_, cx| {
        let bytes = engram_ui::Assets
            .load("themes/gruvbox_dark.json")
            .expect("loading gruvbox_dark.json must not error")
            .expect("gruvbox_dark.json must be present in the embedded asset bundle");
        let theme = engram_theme::Theme::from_json_bytes(&bytes)
            .expect("gruvbox_dark.json must parse as a Theme");
        engram_theme::ThemeRegistry::global_mut(cx).insert(theme);
        engram_theme::activate_theme("Gruvbox Dark", cx)
            .expect("Gruvbox Dark must be registered after insert");
        v_flex()
            .gap(Spacing::Small.pixels())
            .child(Banner::new(Severity::Info, "Info from JSON theme"))
            .child(Banner::new(Severity::Success, "Success from JSON theme"))
            .child(Banner::new(Severity::Warning, "Warning from JSON theme"))
            .child(Banner::new(Severity::Error, "Error from JSON theme"))
            .child(Indicator::dot().color(engram_theme::Color::Accent))
            .into_any_element()
    });
}

// Silence unused-import warnings if a test removes its last reference.
#[allow(dead_code)]
fn _keep_app_alive(_: &mut App) {}
