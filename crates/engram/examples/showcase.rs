//! Live gallery of engram components — every interactive component in the
//! library is wired up to real state, so you can click checkboxes, flip
//! switches, expand the disclosure, and select list rows.
//!
//! Run with: `cargo run --example showcase -p engram`.

use std::cell::Cell;
use std::rc::Rc;

use engram::prelude::*;
use engram::theme::hot_reload::ThemeWatcher;
use gpui::{
    App, AppContext, Bounds, Context, Entity, FocusHandle, InteractiveElement, IntoElement,
    ParentElement, Pixels, Render, SharedString, StatefulInteractiveElement, Styled, Subscription,
    WeakEntity, Window, WindowBounds, WindowOptions, canvas, div, prelude::FluentBuilder, px, size,
};
use gpui_platform::application;

struct Showcase {
    // Currently active theme name. Mirrors the active theme registered in
    // `ThemeRegistry`; updated whenever the user picks a theme from the
    // header bar (which also drops the system-appearance subscription).
    selected_theme: SharedString,
    // Checkboxes (one per size + a tri-state)
    checkbox_small: ToggleState,
    checkbox_default: ToggleState,
    checkbox_large: ToggleState,
    checkbox_tri: ToggleState,
    // Switches
    switch_notifications: ToggleState,
    switch_autosave: ToggleState,
    switch_telemetry: ToggleState,
    // Buttons that can be toggled on click
    button_pin_toggled: bool,
    icon_button_starred: bool,
    // Disclosure expanded state
    disclosure_open: bool,
    // Selected list item id
    selected_nav: SharedString,
    // Selected tab index in the TabBar demo
    selected_tab: usize,
    // Banner dismissed?
    banner_visible: bool,
    // Modal open? And the focus handle it uses while visible (so Escape
    // and backdrop clicks route to it).
    modal_open: bool,
    modal_focus: FocusHandle,
    // Menu open + the captured trigger button bounds (set by a `canvas`
    // overlay during prepaint, read on the next render). The menu is a
    // stateful entity that owns its own focus handle and emits
    // `DismissEvent` — we subscribe below to flip `menu_open` back off.
    menu_open: bool,
    menu: Entity<Menu>,
    menu_trigger_bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
    // Last submitted value from the text field
    submitted_value: SharedString,
    // The text field entity
    text_field: Entity<TextField>,
    // System appearance observer — kept alive to mirror OS dark/light
    // onto the active theme.
    _appearance_sub: Option<Subscription>,
    // Live hot-reload watcher for the JSON theme directory. Edits to
    // `crates/engram-ui/assets/themes/*.json` show up on the next frame.
    _theme_watcher: Option<ThemeWatcher>,
}

impl Showcase {
    fn new(cx: &mut Context<Self>) -> Self {
        let weak = cx.entity().downgrade();
        let text_field = cx.new(|cx| {
            TextField::with_value(cx, "Hello, engram")
                .placeholder("Type something…")
                .on_submit(move |value, _window, cx| {
                    let value = SharedString::from(value.to_string());
                    weak.update(cx, |this, cx| {
                        this.submitted_value = value;
                        cx.notify();
                    })
                    .ok();
                })
        });
        // Build the menu entity once — its contents are static, so there's
        // no reason to reconstruct it on every render. Each entry's handler
        // is a plain no-op: the menu emits `DismissEvent` automatically
        // after invoking an entry, and the subscription below flips the
        // `menu_open` flag back off.
        let menu = cx.new(|cx| {
            Menu::new(cx)
                .header("File")
                .entry_with_icon("menu-new", IconName::Plus, "New File", |_, _, _| {})
                .keybinding_entry("menu-save", "Save", ["Ctrl", "S"], |_, _, _| {})
                .keybinding_entry(
                    "menu-saveas",
                    "Save As…",
                    ["Ctrl", "Shift", "S"],
                    |_, _, _| {},
                )
                .separator()
                .header("Edit")
                .entry("menu-cut", "Cut", |_, _, _| {})
                .entry("menu-copy", "Copy", |_, _, _| {})
                .entry("menu-paste", "Paste", |_, _, _| {})
                .separator()
                .disabled_entry("menu-disabled", "Unavailable")
        });
        cx.subscribe(&menu, |this, _, _: &gpui::DismissEvent, cx| {
            this.menu_open = false;
            cx.notify();
        })
        .detach();
        Self {
            selected_theme: cx.theme().name.clone(),
            checkbox_small: ToggleState::Selected,
            checkbox_default: ToggleState::Unselected,
            checkbox_large: ToggleState::Selected,
            checkbox_tri: ToggleState::Indeterminate,
            switch_notifications: ToggleState::Selected,
            switch_autosave: ToggleState::Unselected,
            switch_telemetry: ToggleState::Selected,
            button_pin_toggled: false,
            icon_button_starred: true,
            disclosure_open: true,
            selected_nav: SharedString::from("nav-search"),
            selected_tab: 0,
            banner_visible: true,
            modal_open: false,
            modal_focus: cx.focus_handle(),
            menu_open: false,
            menu,
            menu_trigger_bounds: Rc::new(Cell::new(None)),
            submitted_value: SharedString::default(),
            text_field,
            _appearance_sub: None,
            _theme_watcher: None,
        }
    }
}

/// Build a `ToggleState` click handler that mutates a single field on
/// `Showcase` and re-renders. Used by every Checkbox / Switch in the
/// showcase.
fn toggle_setter<F>(
    weak: &WeakEntity<Showcase>,
    set: F,
) -> impl Fn(&ToggleState, &mut Window, &mut App) + 'static
where
    F: Fn(&mut Showcase, ToggleState) + 'static,
{
    let weak = weak.clone();
    move |state, _window, cx| {
        let state = *state;
        weak.update(cx, |this, cx| {
            set(this, state);
            cx.notify();
        })
        .ok();
    }
}

/// Click handler that flips a `bool` field on `Showcase`. Used for the
/// toggleable Button / IconButton demos.
fn bool_toggle<F>(
    weak: &WeakEntity<Showcase>,
    pick: F,
) -> impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static
where
    F: Fn(&mut Showcase) -> &mut bool + 'static,
{
    let weak = weak.clone();
    move |_event, _window, cx| {
        weak.update(cx, |this, cx| {
            let field = pick(this);
            *field = !*field;
            cx.notify();
        })
        .ok();
    }
}

/// Click handler that sets a fixed value on a `Showcase` field of type `T`.
fn set_field<T, F>(
    weak: &WeakEntity<Showcase>,
    value: T,
    pick: F,
) -> impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static
where
    T: Clone + 'static,
    F: Fn(&mut Showcase) -> &mut T + 'static,
{
    let weak = weak.clone();
    move |_event, _window, cx| {
        let value = value.clone();
        weak.update(cx, |this, cx| {
            *pick(this) = value;
            cx.notify();
        })
        .ok();
    }
}

impl Render for Showcase {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors();
        let weak = cx.entity().downgrade();

        // Pull every theme registered in the global registry — built-in
        // engram + any JSON themes loaded from `Assets` in `main()`. The
        // selector below renders one button per entry.
        let theme_names = engram::theme::ThemeRegistry::global(cx).names();
        let selected_theme = self.selected_theme.clone();

        v_flex()
            .id("showcase-root")
            .size_full()
            .bg(colors.background)
            .p(Spacing::XXLarge.pixels())
            .gap(Spacing::Large.pixels())
            .overflow_y_scroll()
            // -------------------- Header --------------------
            .child(
                h_flex()
                    .w_full()
                    .items_center()
                    .justify_between()
                    .child(
                        Headline::new("engram showcase")
                            .size(HeadlineSize::Medium),
                    )
                    .child(
                        h_flex()
                            .gap(Spacing::Small.pixels())
                            .children(theme_names.into_iter().map(|name| {
                                let is_current = name == selected_theme;
                                let id =
                                    SharedString::from(format!("btn-theme-{name}"));
                                let label = name.clone();
                                let weak = weak.clone();
                                let target = name.clone();
                                Button::new(id, label)
                                    .style(if is_current {
                                        ButtonStyle::Filled
                                    } else {
                                        ButtonStyle::Subtle
                                    })
                                    .toggle_state(is_current)
                                    .on_click(move |_event, _window, cx| {
                                        let target = target.clone();
                                        weak.update(cx, |this, cx| {
                                            // Stop mirroring the OS appearance
                                            // so an explicit user pick sticks.
                                            this._appearance_sub = None;
                                            if engram::theme::activate_theme(&target, cx)
                                                .is_ok()
                                            {
                                                this.selected_theme = target;
                                                cx.notify();
                                            }
                                        })
                                        .ok();
                                    })
                            })),
                    ),
            )
            // -------------------- Typography --------------------
            .child(section(
                "Label sizes",
                h_flex()
                    .gap(Spacing::Large.pixels())
                    .items_baseline()
                    .child(Label::new("XSmall").size(LabelSize::XSmall))
                    .child(Label::new("Small").size(LabelSize::Small))
                    .child(Label::new("Default"))
                    .child(Label::new("Large").size(LabelSize::Large)),
            ))
            .child(section(
                "Label colors",
                h_flex()
                    .gap(Spacing::Large.pixels())
                    .child(Label::new("Default"))
                    .child(Label::new("Muted").color(Color::Muted))
                    .child(Label::new("Accent").color(Color::Accent))
                    .child(Label::new("Success").color(Color::Success))
                    .child(Label::new("Warning").color(Color::Warning))
                    .child(Label::new("Error").color(Color::Error))
                    .child(Label::new("Disabled").color(Color::Disabled)),
            ))
            .child(section(
                "Label modifiers",
                h_flex()
                    .gap(Spacing::Large.pixels())
                    .items_baseline()
                    .child(Label::new("Bold").weight(gpui::FontWeight::BOLD))
                    .child(Label::new("Italic").italic())
                    .child(Label::new("Underline").underline())
                    .child(Label::new("Strikethrough").strikethrough())
                    .child(Label::new("Faded 50%").alpha(0.5)),
            ))
            .child(section(
                "Headline sizes",
                v_flex()
                    .gap(Spacing::XSmall.pixels())
                    .child(Headline::new("XSmall headline").size(HeadlineSize::XSmall))
                    .child(Headline::new("Small headline").size(HeadlineSize::Small))
                    .child(Headline::new("Medium headline (default)"))
                    .child(Headline::new("Large headline").size(HeadlineSize::Large))
                    .child(Headline::new("XLarge headline").size(HeadlineSize::XLarge)),
            ))
            // -------------------- Icons --------------------
            .child(section(
                "Icons",
                h_flex()
                    .gap(Spacing::Large.pixels())
                    .child(Icon::new(IconName::Check))
                    .child(Icon::new(IconName::Close))
                    .child(Icon::new(IconName::ChevronDown))
                    .child(Icon::new(IconName::ChevronRight))
                    .child(Icon::new(IconName::Plus))
                    .child(Icon::new(IconName::Dash))
                    .child(Icon::new(IconName::MagnifyingGlass))
                    .child(Icon::new(IconName::Settings))
                    .child(Icon::new(IconName::Warning).color(Color::Warning))
                    .child(Icon::new(IconName::Info).color(Color::Info))
                    .child(Icon::new(IconName::XCircle).color(Color::Error))
                    .child(Icon::new(IconName::Check).color(Color::Success)),
            ))
            .child(section(
                "Icon sizes",
                h_flex()
                    .gap(Spacing::Large.pixels())
                    .items_center()
                    .child(Icon::new(IconName::Settings).size(IconSize::XSmall))
                    .child(Icon::new(IconName::Settings).size(IconSize::Small))
                    .child(Icon::new(IconName::Settings).size(IconSize::Medium))
                    .child(Icon::new(IconName::Settings).size(IconSize::Large)),
            ))
            .child(Divider::horizontal())
            // -------------------- Buttons --------------------
            .child(section(
                "Button styles",
                h_flex()
                    .gap(Spacing::Medium.pixels())
                    .child(Button::new("btn-filled", "Filled").style(ButtonStyle::Filled))
                    .child(Button::new("btn-subtle", "Subtle").style(ButtonStyle::Subtle))
                    .child(Button::new("btn-outlined", "Outlined").style(ButtonStyle::Outlined))
                    .child(
                        Button::new("btn-outlined-ghost", "Outlined ghost")
                            .style(ButtonStyle::OutlinedGhost),
                    )
                    .child(
                        Button::new("btn-transparent", "Transparent")
                            .style(ButtonStyle::Transparent),
                    ),
            ))
            .child(section(
                "Button tints",
                h_flex()
                    .gap(Spacing::Medium.pixels())
                    .child(
                        Button::new("btn-tint-accent", "Accent")
                            .style(ButtonStyle::Tinted(TintColor::Accent)),
                    )
                    .child(
                        Button::new("btn-tint-success", "Success")
                            .style(ButtonStyle::Tinted(TintColor::Success)),
                    )
                    .child(
                        Button::new("btn-tint-warning", "Warning")
                            .style(ButtonStyle::Tinted(TintColor::Warning)),
                    )
                    .child(
                        Button::new("btn-tint-error", "Error")
                            .style(ButtonStyle::Tinted(TintColor::Error)),
                    ),
            ))
            .child(section(
                "Button sizes",
                h_flex()
                    .gap(Spacing::Medium.pixels())
                    .items_center()
                    .child(Button::new("btn-cmp", "Compact").size(ButtonSize::Compact))
                    .child(Button::new("btn-def", "Default"))
                    .child(Button::new("btn-lg", "Large").size(ButtonSize::Large)),
            ))
            .child(section(
                "Button extras (Pin button toggles on click)",
                h_flex()
                    .gap(Spacing::Medium.pixels())
                    .child(Button::new("btn-icon", "Save").icon(IconName::Check))
                    .child(Button::new("btn-disabled", "Disabled").disabled(true))
                    .child(
                        Button::new("btn-pin", if self.button_pin_toggled { "Pinned" } else { "Pin" })
                            .icon(IconName::Pin)
                            .style(ButtonStyle::Subtle)
                            .toggle_state(self.button_pin_toggled)
                            .on_click(bool_toggle(&weak, |this| &mut this.button_pin_toggled)),
                    ),
            ))
            .child(section(
                "IconButton (Star toggles on click)",
                h_flex()
                    .gap(Spacing::Medium.pixels())
                    .child(IconButton::new("ib-filled", IconName::Settings))
                    .child(
                        IconButton::new("ib-ghost", IconName::MagnifyingGlass)
                            .style(ButtonStyle::Subtle),
                    )
                    .child(IconButton::new("ib-out", IconName::Plus).style(ButtonStyle::Outlined))
                    .child(
                        IconButton::new(
                            "ib-star",
                            if self.icon_button_starred {
                                IconName::StarFilled
                            } else {
                                IconName::Star
                            },
                        )
                        .style(ButtonStyle::Subtle)
                        .toggle_state(self.icon_button_starred)
                        .on_click(bool_toggle(&weak, |this| &mut this.icon_button_starred)),
                    )
                    .child(IconButton::new("ib-disabled", IconName::Close).disabled(true)),
            ))
            .child(Divider::horizontal())
            // -------------------- Checkboxes (sizes + states + tri-state, all interactive) --------------------
            .child(section(
                "Checkbox (every size is clickable)",
                h_flex()
                    .gap(Spacing::Large.pixels())
                    .items_center()
                    .child(
                        Checkbox::new("cb-sm", self.checkbox_small)
                            .size(CheckboxSize::Small)
                            .label("Small")
                            .on_click(toggle_setter(&weak, |this, s| this.checkbox_small = s)),
                    )
                    .child(
                        Checkbox::new("cb-def", self.checkbox_default)
                            .label("Default")
                            .on_click(toggle_setter(&weak, |this, s| this.checkbox_default = s)),
                    )
                    .child(
                        Checkbox::new("cb-lg", self.checkbox_large)
                            .size(CheckboxSize::Large)
                            .label("Large")
                            .on_click(toggle_setter(&weak, |this, s| this.checkbox_large = s)),
                    )
                    .child(
                        Checkbox::new("cb-tri", self.checkbox_tri)
                            .label("Tri-state")
                            .on_click(toggle_setter(&weak, |this, s| this.checkbox_tri = s)),
                    )
                    .child(
                        Checkbox::new("cb-disabled", true)
                            .label("Disabled")
                            .disabled(true),
                    ),
            ))
            // -------------------- Switches (interactive) --------------------
            .child(section(
                "Switch",
                h_flex()
                    .gap(Spacing::Large.pixels())
                    .items_center()
                    .child(
                        Switch::new("sw-notif", self.switch_notifications)
                            .label("Notifications")
                            .on_click(toggle_setter(&weak, |this, s| {
                                this.switch_notifications = s
                            })),
                    )
                    .child(
                        Switch::new("sw-auto", self.switch_autosave)
                            .label("Auto-save")
                            .on_click(toggle_setter(&weak, |this, s| this.switch_autosave = s)),
                    )
                    .child(
                        Switch::new("sw-tel", self.switch_telemetry)
                            .label("Telemetry")
                            .on_click(toggle_setter(&weak, |this, s| this.switch_telemetry = s)),
                    )
                    .child(
                        Switch::new("sw-disabled", false)
                            .label("Disabled")
                            .disabled(true),
                    ),
            ))
            // -------------------- Tooltips --------------------
            .child(section(
                "Tooltip (hover the buttons)",
                h_flex()
                    .gap(Spacing::Medium.pixels())
                    .child(
                        Button::new("btn-tip-text", "Hover me")
                            .tooltip(Tooltip::text("This is a tooltip")),
                    )
                    .child(
                        Button::new("btn-tip-meta", "With meta")
                            .style(ButtonStyle::Subtle)
                            .tooltip(Tooltip::with_meta("Save file", "Ctrl+S")),
                    )
                    .child(
                        IconButton::new("ib-tip", IconName::Settings)
                            .style(ButtonStyle::Subtle)
                            .tooltip(Tooltip::text("Settings")),
                    ),
            ))
            .child(Divider::horizontal())
            // -------------------- List (selectable) --------------------
            .child(section(
                "List (click to select)",
                v_flex().w(px(280.0)).child(
                    List::new()
                        .header("Navigation")
                        .child(self.nav_item("nav-home", IconName::ChevronRight, "Home", &weak))
                        .child(self.nav_item(
                            "nav-search",
                            IconName::MagnifyingGlass,
                            "Search",
                            &weak,
                        ))
                        .child(self.nav_item(
                            "nav-settings",
                            IconName::Settings,
                            "Settings",
                            &weak,
                        ))
                        .child(
                            ListItem::new("nav-disabled")
                                .start_slot(Icon::new(IconName::Close))
                                .child(Label::new("Unavailable"))
                                .disabled(true),
                        ),
                ),
            ))
            .child(section(
                "Empty list",
                v_flex()
                    .w(px(280.0))
                    .child(List::new().header("Recent").empty_message("No recent items")),
            ))
            // -------------------- Tree list (Phase 8 fields) --------------------
            .child(section(
                "Tree list (indent_level + spacing + inset)",
                v_flex().w(px(320.0)).child(
                    List::new()
                        .header("Project")
                        .child(
                            ListItem::new("tree-src")
                                .spacing(ListItemSpacing::Dense)
                                .inset(true)
                                .indent_level(0)
                                .start_slot(Icon::new(IconName::Folder))
                                .child(Label::new("src")),
                        )
                        .child(
                            ListItem::new("tree-src-main")
                                .spacing(ListItemSpacing::Dense)
                                .inset(true)
                                .indent_level(1)
                                .start_slot(Icon::new(IconName::File))
                                .child(Label::new("main.rs")),
                        )
                        .child(
                            ListItem::new("tree-src-lib")
                                .spacing(ListItemSpacing::Dense)
                                .inset(true)
                                .indent_level(1)
                                .start_slot(Icon::new(IconName::Folder))
                                .child(Label::new("lib")),
                        )
                        .child(
                            ListItem::new("tree-src-lib-mod")
                                .spacing(ListItemSpacing::Dense)
                                .inset(true)
                                .indent_level(2)
                                .start_slot(Icon::new(IconName::File))
                                .child(Label::new("mod.rs"))
                                .end_slot(Icon::new(IconName::Trash))
                                .show_end_slot_on_hover(),
                        )
                        .child(
                            ListItem::new("tree-src-lib-utils")
                                .spacing(ListItemSpacing::Dense)
                                .inset(true)
                                .indent_level(2)
                                .start_slot(Icon::new(IconName::File))
                                .child(Label::new("utils.rs")),
                        )
                        .child(
                            ListItem::new("tree-cargo")
                                .spacing(ListItemSpacing::Dense)
                                .inset(true)
                                .indent_level(0)
                                .start_slot(Icon::new(IconName::File))
                                .child(Label::new("Cargo.toml")),
                        ),
                ),
            ))
            .child(Divider::horizontal())
            // -------------------- Indicators --------------------
            .child(section(
                "Indicator",
                h_flex()
                    .gap(Spacing::Large.pixels())
                    .items_center()
                    .child(Indicator::dot())
                    .child(Indicator::dot().color(Color::Success))
                    .child(Indicator::dot().color(Color::Warning))
                    .child(Indicator::dot().color(Color::Error))
                    .child(
                        v_flex()
                            .w(px(48.0))
                            .child(Indicator::bar().color(Color::Accent)),
                    )
                    .child(Indicator::icon(Icon::new(IconName::Check)).color(Color::Success))
                    .child(Indicator::icon(Icon::new(IconName::Close)).color(Color::Error)),
            ))
            // -------------------- Disclosure (interactive) --------------------
            .child(section(
                "Disclosure (click to expand)",
                v_flex()
                    .gap(Spacing::Small.pixels())
                    .child(
                        h_flex()
                            .gap(Spacing::XSmall.pixels())
                            .items_center()
                            .child(
                                Disclosure::new("disc-1", self.disclosure_open).on_click(
                                    bool_toggle(&weak, |this| &mut this.disclosure_open),
                                ),
                            )
                            .child(Label::new("Advanced settings")),
                    )
                    .when(self.disclosure_open, |this| {
                        this.child(
                            v_flex()
                                .pl(px(24.0))
                                .gap(Spacing::XSmall.pixels())
                                .child(Label::new("Setting one").color(Color::Muted))
                                .child(Label::new("Setting two").color(Color::Muted))
                                .child(Label::new("Setting three").color(Color::Muted)),
                        )
                    })
                    .child(
                        h_flex()
                            .gap(Spacing::XSmall.pixels())
                            .items_center()
                            .child(Disclosure::new("disc-disabled", false).disabled(true))
                            .child(Label::new("Disabled section").color(Color::Disabled)),
                    ),
            ))
            // -------------------- KeyBinding --------------------
            .child(section(
                "KeyBinding",
                h_flex()
                    .gap(Spacing::Large.pixels())
                    .items_center()
                    .child(KeyBinding::new(["Cmd", "S"]))
                    .child(KeyBinding::new(["Ctrl", "Shift", "P"]))
                    .child(KeyBinding::new(["Esc"])),
            ))
            .child(Divider::horizontal())
            .child(section(
                "Divider",
                v_flex()
                    .gap(Spacing::Medium.pixels())
                    .w(px(320.0))
                    .child(Label::new("Above").color(Color::Muted))
                    .child(Divider::horizontal())
                    .child(Label::new("Below").color(Color::Muted)),
            ))
            .child(Divider::horizontal())
            // -------------------- TabBar --------------------
            .child(section(
                "TabBar (click to select)",
                v_flex()
                    .gap(Spacing::Small.pixels())
                    .child(
                        TabBar::new()
                            .child(self.tab("tab-overview", IconName::Eye, "Overview", 0, &weak))
                            .child(self.tab("tab-files", IconName::File, "Files", 1, &weak))
                            .child(self.tab(
                                "tab-settings",
                                IconName::Settings,
                                "Settings",
                                2,
                                &weak,
                            )),
                    )
                    .child(
                        Label::new(match self.selected_tab {
                            0 => "Overview content",
                            1 => "Files content",
                            _ => "Settings content",
                        })
                        .color(Color::Muted),
                    ),
            ))
            // -------------------- Avatar / Facepile --------------------
            .child(section(
                "Avatar / Facepile",
                h_flex()
                    .gap(Spacing::Large.pixels())
                    .items_center()
                    .child(Avatar::new("Ada").size(AvatarSize::Small))
                    .child(Avatar::new("Linus"))
                    .child(Avatar::new("Grace").size(AvatarSize::Large))
                    .child(
                        Facepile::new()
                            .push(Avatar::new("Ada"))
                            .push(Avatar::new("Linus"))
                            .push(Avatar::new("Grace"))
                            .push(Avatar::new("Donald")),
                    ),
            ))
            // -------------------- Chip / CountBadge --------------------
            .child(section(
                "Chip / CountBadge",
                h_flex()
                    .gap(Spacing::Medium.pixels())
                    .items_center()
                    .child(Chip::new("Default"))
                    .child(Chip::new("Accent").style(ChipStyle::Accent))
                    .child(Chip::new("Success").style(ChipStyle::Success))
                    .child(Chip::new("Warning").style(ChipStyle::Warning))
                    .child(Chip::new("Error").style(ChipStyle::Error))
                    .child(CountBadge::new(3))
                    .child(CountBadge::new(42))
                    .child(CountBadge::new(150)),
            ))
            .child(Divider::horizontal())
            // -------------------- Banner / Notification --------------------
            .child(section(
                "Banner",
                v_flex()
                    .gap(Spacing::Small.pixels())
                    .when(self.banner_visible, |this| {
                        this.child(
                            Banner::new(Severity::Info, "New version available")
                                .description("Engram 0.2 is ready to install.")
                                .action(
                                    Button::new("banner-update", "Update")
                                        .style(ButtonStyle::Tinted(TintColor::Accent))
                                        .size(ButtonSize::Compact),
                                )
                                .on_dismiss({
                                    let weak = weak.clone();
                                    move |_, _, cx| {
                                        weak.update(cx, |this, cx| {
                                            this.banner_visible = false;
                                            cx.notify();
                                        })
                                        .ok();
                                    }
                                }),
                        )
                    })
                    .when(!self.banner_visible, |this| {
                        this.child(
                            Button::new("banner-restore", "Restore banner")
                                .style(ButtonStyle::Subtle)
                                .size(ButtonSize::Compact)
                                .on_click(set_field(&weak, true, |this| &mut this.banner_visible)),
                        )
                    })
                    .child(Banner::new(Severity::Success, "All checks passed"))
                    .child(
                        Banner::new(Severity::Warning, "Disk usage at 90%")
                            .description("Consider freeing up some space."),
                    )
                    .child(
                        Banner::new(Severity::Error, "Build failed")
                            .description("3 tests failed in `engram-ui`."),
                    ),
            ))
            .child(section(
                "Notification (toast style)",
                h_flex()
                    .gap(Spacing::Medium.pixels())
                    .items_start()
                    .child(
                        Notification::new(Severity::Success, "Saved")
                            .description("Your changes were saved automatically."),
                    )
                    .child(
                        Notification::new(Severity::Error, "Sync failed")
                            .description("Check your network connection and retry."),
                    ),
            ))
            .child(Divider::horizontal())
            // -------------------- Menu (real anchored popover) --------------------
            .child(section(
                "Menu (click to open — keyboard-navigable)",
                {
                    let bounds_slot = self.menu_trigger_bounds.clone();
                    let menu_entity = self.menu.clone();
                    let open_menu_handler = {
                        let weak = weak.clone();
                        move |_event: &gpui::ClickEvent, window: &mut Window, cx: &mut App| {
                            weak.update(cx, |this, cx| {
                                this.menu_open = !this.menu_open;
                                if this.menu_open {
                                    // Focus the menu entity's own focus
                                    // handle so arrow keys, Enter, and Esc
                                    // dispatch through its key context.
                                    let handle = this.menu.read(cx).focus_handle().clone();
                                    window.focus(&handle, cx);
                                }
                                cx.notify();
                            })
                            .ok();
                        }
                    };
                    let trigger_button = Button::new("btn-menu-trigger", "Open menu")
                        .icon(IconName::ChevronDown)
                        .style(ButtonStyle::Outlined)
                        .on_click(open_menu_handler);
                    // Wrap the trigger in a relatively-positioned container
                    // and overlay a `canvas` that captures the trigger's
                    // bounds. The canvas paint callback is empty — we only
                    // need its prepaint hook to grab `bounds`.
                    let trigger_with_capture = div()
                        .relative()
                        .child(trigger_button)
                        .child(
                            canvas(
                                move |bounds, _window, _cx| {
                                    bounds_slot.set(Some(bounds));
                                },
                                |_, _, _, _| {},
                            )
                            .absolute()
                            .inset_0()
                            .size_full(),
                        );

                    let menu_open = self.menu_open;
                    let trigger_bounds = self.menu_trigger_bounds.get();
                    let anchor_focus = self.menu.read(cx).focus_handle().clone();
                    let weak_for_dismiss = weak.clone();

                    v_flex()
                        .gap(Spacing::Small.pixels())
                        .child(trigger_with_capture)
                        .when(menu_open, |this| {
                            let Some(bounds) = trigger_bounds else {
                                return this;
                            };
                            this.child(anchored_popover(
                                anchor_focus,
                                gpui::Corner::TopLeft,
                                bounds,
                                menu_entity,
                                move |_window, cx| {
                                    weak_for_dismiss
                                        .update(cx, |this, cx| {
                                            this.menu_open = false;
                                            cx.notify();
                                        })
                                        .ok();
                                },
                            ))
                        })
                },
            ))
            // -------------------- Modal --------------------
            .child(section(
                "Modal",
                v_flex()
                    .gap(Spacing::Small.pixels())
                    .child(
                        Button::new("btn-open-modal", "Open modal")
                            .style(ButtonStyle::Tinted(TintColor::Accent))
                            .on_click({
                                let weak = weak.clone();
                                move |_event, window, cx| {
                                    weak.update(cx, |this, cx| {
                                        this.modal_open = true;
                                        // Focus the modal's handle so Escape
                                        // and backdrop clicks route here.
                                        window.focus(&this.modal_focus, cx);
                                        cx.notify();
                                    })
                                    .ok();
                                }
                            }),
                    ),
            ))
            // -------------------- TextField --------------------
            .child(section(
                "TextField (click to focus, type, Enter to submit)",
                v_flex()
                    .gap(Spacing::Small.pixels())
                    .child(self.text_field.clone())
                    .child(
                        Label::new(if self.submitted_value.is_empty() {
                            SharedString::from("Last submitted: (none yet)")
                        } else {
                            format!("Last submitted: {}", self.submitted_value).into()
                        })
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                    ),
            ))
            // -------------------- Modal overlay (rendered last so it floats above) --------------------
            .when(self.modal_open, |this| {
                let weak_for_modal = weak.clone();
                let weak_for_buttons = weak.clone();
                this.child(modal_overlay(
                    self.modal_focus.clone(),
                    Modal::new()
                        .title("Delete file?")
                        .child(
                            Label::new("This action cannot be undone.")
                                .color(Color::Muted),
                        )
                        .footer(
                            h_flex()
                                .gap(Spacing::Small.pixels())
                                .justify_end()
                                .child(
                                    Button::new("modal-cancel", "Cancel")
                                        .style(ButtonStyle::Subtle)
                                        .on_click(set_field(
                                            &weak_for_buttons,
                                            false,
                                            |this| &mut this.modal_open,
                                        )),
                                )
                                .child(
                                    Button::new("modal-delete", "Delete")
                                        .style(ButtonStyle::Tinted(TintColor::Accent))
                                        .on_click(set_field(
                                            &weak_for_buttons,
                                            false,
                                            |this| &mut this.modal_open,
                                        )),
                                ),
                        ),
                    move |_window, cx| {
                        weak_for_modal
                            .update(cx, |this, cx| {
                                this.modal_open = false;
                                cx.notify();
                            })
                            .ok();
                    },
                ))
            })
    }
}

impl Showcase {
    /// Build a selectable navigation row whose `selected` state and click
    /// handler are wired to `self.selected_nav`.
    fn nav_item(
        &self,
        id: &'static str,
        icon: IconName,
        label: &'static str,
        weak: &WeakEntity<Self>,
    ) -> ListItem {
        let id_owned = SharedString::from(id);
        let is_selected = self.selected_nav == id_owned;
        let weak = weak.clone();
        ListItem::new(id)
            .start_slot(Icon::new(icon))
            .child(Label::new(label))
            .toggle_state(is_selected)
            .on_click(move |_event, _window, cx| {
                let id = id_owned.clone();
                weak.update(cx, |this, cx| {
                    this.selected_nav = id;
                    cx.notify();
                })
                .ok();
            })
    }

    /// Build a TabBar tab whose selection is driven by `self.selected_tab`.
    fn tab(
        &self,
        id: &'static str,
        icon: IconName,
        label: &'static str,
        index: usize,
        weak: &WeakEntity<Self>,
    ) -> Tab {
        let weak = weak.clone();
        Tab::new(id, label)
            .icon(icon)
            .toggle_state(self.selected_tab == index)
            .on_click(move |_event, _window, cx| {
                weak.update(cx, |this, cx| {
                    this.selected_tab = index;
                    cx.notify();
                })
                .ok();
            })
    }
}


/// Walk every JSON file under `themes/` in the embedded `Assets` source,
/// parse it as a [`engram::theme::Theme`], and insert it into the global
/// [`engram::theme::ThemeRegistry`]. The built-in `Engram Dark` /
/// `Engram Light` already live there from `engram::theme::init`, so this
/// only adds the JSON-shipped extras (gruvbox at the moment).
fn register_embedded_themes(cx: &mut App) {
    use engram::theme::{Theme, ThemeRegistry};
    use gpui::AssetSource;

    let asset_paths = match Assets.list("themes/") {
        Ok(paths) => paths,
        Err(err) => {
            eprintln!("engram showcase: failed to list embedded themes: {err}");
            return;
        }
    };

    for path in asset_paths {
        if !path.ends_with(".json") {
            continue;
        }
        match Assets.load(&path) {
            Ok(Some(bytes)) => match Theme::from_json_bytes(&bytes) {
                Ok(theme) => {
                    ThemeRegistry::global_mut(cx).insert(theme);
                }
                Err(err) => {
                    eprintln!("engram showcase: failed to parse {path}: {err}");
                }
            },
            Ok(None) => {}
            Err(err) => {
                eprintln!("engram showcase: failed to load {path}: {err}");
            }
        }
    }
}

/// Small helper for building a titled section in the showcase.
fn section(title: &'static str, body: impl IntoElement) -> impl IntoElement {
    v_flex()
        .gap(Spacing::Small.pixels())
        .child(
            Label::new(title)
                .size(LabelSize::Small)
                .color(Color::Muted),
        )
        .child(body)
}

fn main() {
    application().with_assets(Assets).run(|cx: &mut App| {
        engram::theme::init(cx);
        engram::ui::init(cx);

        // Load every JSON theme embedded in `engram_ui::Assets` and insert
        // it into the registry, so the header bar selector can list them
        // alongside the built-in defaults. Failures are non-fatal — a bad
        // theme just doesn't appear in the picker.
        register_embedded_themes(cx);

        // Watch the repo's canonical themes directory so edits to the
        // JSON fixtures show up instantly in the showcase.
        let themes_dir = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../engram-ui/assets/themes"
        );
        let mut theme_watcher =
            match engram::theme::hot_reload::watch_themes_dir(themes_dir, cx) {
                Ok(watcher) => Some(watcher),
                Err(err) => {
                    eprintln!("engram showcase: hot reload disabled: {err}");
                    None
                }
            };

        let bounds = Bounds::centered(None, size(px(960.0), px(760.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| {
                // Mirror the OS light/dark appearance onto the active
                // theme from the first frame onwards.
                let appearance_sub = engram::theme::sync_with_system_appearance(
                    Default::default(),
                    window,
                    cx,
                );
                let entity = cx.new(Showcase::new);
                entity.update(cx, |showcase, cx| {
                    showcase.selected_theme = cx.theme().name.clone();
                    showcase._appearance_sub = Some(appearance_sub);
                    showcase._theme_watcher = theme_watcher.take();
                });
                entity
            },
        )
        .unwrap();

        cx.activate(true);
    });
}
