//! Story gallery for engram — a per-component showcase browser with sidebar
//! navigation and theme switching.
//!
//! Run with: `cargo run -p story`

mod layout;
mod stories;

/// Re-exports for story files — each story just writes `use crate::prelude::*`.
pub mod prelude {
    pub use engram::prelude::*;
    pub use gpui::{
        AnyView, App, AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement,
        Render, SharedString, StatefulInteractiveElement, Styled, WeakEntity, Window,
        prelude::FluentBuilder, px,
    };

    pub use crate::layout::{example, example_group};
}

use std::ops::DerefMut;

use prelude::*;
use gpui::{Bounds, Subscription, WindowBounds, WindowOptions, size};
use gpui_platform::application;
use strum::{Display, EnumIter, IntoEnumIterator};

// ---------------------------------------------------------------------------
// Story registry
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, Display)]
pub enum StoryCategory {
    Typography,
    #[strum(serialize = "Icons & Images")]
    IconsAndImages,
    Buttons,
    Inputs,
    #[strum(serialize = "Data Display")]
    DataDisplay,
    Feedback,
    Navigation,
    Layout,
}

pub struct StoryEntry {
    pub name: &'static str,
    pub category: StoryCategory,
    pub build: fn(&mut Window, &mut App) -> AnyView,
}

pub static STORIES: &[StoryEntry] = &[
    // Typography
    StoryEntry { name: "Label", category: StoryCategory::Typography, build: stories::label::build },
    StoryEntry { name: "Headline", category: StoryCategory::Typography, build: stories::headline::build },
    StoryEntry { name: "HighlightedLabel", category: StoryCategory::Typography, build: stories::highlighted_label::build },
    // Icons & Images
    StoryEntry { name: "Icon", category: StoryCategory::IconsAndImages, build: stories::icon::build },
    StoryEntry { name: "DecoratedIcon", category: StoryCategory::IconsAndImages, build: stories::decorated_icon::build },
    StoryEntry { name: "Image", category: StoryCategory::IconsAndImages, build: stories::image::build },
    StoryEntry { name: "Avatar", category: StoryCategory::IconsAndImages, build: stories::avatar::build },
    // Buttons
    StoryEntry { name: "Button", category: StoryCategory::Buttons, build: stories::button::build },
    StoryEntry { name: "IconButton", category: StoryCategory::Buttons, build: stories::icon_button::build },
    StoryEntry { name: "ButtonLink", category: StoryCategory::Buttons, build: stories::button_link::build },
    StoryEntry { name: "SplitButton", category: StoryCategory::Buttons, build: stories::split_button::build },
    StoryEntry { name: "CopyButton", category: StoryCategory::Buttons, build: stories::copy_button::build },
    StoryEntry { name: "ToggleButtonGroup", category: StoryCategory::Buttons, build: stories::toggle_button::build },
    // Inputs
    StoryEntry { name: "Checkbox", category: StoryCategory::Inputs, build: stories::checkbox::build },
    StoryEntry { name: "Switch", category: StoryCategory::Inputs, build: stories::switch::build },
    StoryEntry { name: "TextField", category: StoryCategory::Inputs, build: stories::text_field::build },
    StoryEntry { name: "Disclosure", category: StoryCategory::Inputs, build: stories::disclosure::build },
    StoryEntry { name: "DropdownMenu", category: StoryCategory::Inputs, build: stories::dropdown_menu::build },
    // Data Display
    StoryEntry { name: "List", category: StoryCategory::DataDisplay, build: stories::list::build },
    StoryEntry { name: "TreeView", category: StoryCategory::DataDisplay, build: stories::tree_view::build },
    StoryEntry { name: "Progress", category: StoryCategory::DataDisplay, build: stories::progress::build },
    StoryEntry { name: "Indicator", category: StoryCategory::DataDisplay, build: stories::indicator::build },
    StoryEntry { name: "Chip", category: StoryCategory::DataDisplay, build: stories::chip::build },
    StoryEntry { name: "KeyBinding", category: StoryCategory::DataDisplay, build: stories::keybinding::build },
    StoryEntry { name: "KeybindingHint", category: StoryCategory::DataDisplay, build: stories::keybinding_hint::build },
    // Feedback
    StoryEntry { name: "Banner", category: StoryCategory::Feedback, build: stories::banner::build },
    StoryEntry { name: "Callout", category: StoryCategory::Feedback, build: stories::callout::build },
    StoryEntry { name: "Spinner", category: StoryCategory::Feedback, build: stories::spinner::build },
    StoryEntry { name: "Tooltip", category: StoryCategory::Feedback, build: stories::tooltip::build },
    // Navigation
    StoryEntry { name: "Tab", category: StoryCategory::Navigation, build: stories::tab::build },
    StoryEntry { name: "Menu", category: StoryCategory::Navigation, build: stories::menu::build },
    // Layout
    StoryEntry { name: "Divider", category: StoryCategory::Layout, build: stories::divider::build },
    StoryEntry { name: "GradientFade", category: StoryCategory::Layout, build: stories::gradient_fade::build },
    StoryEntry { name: "Modal", category: StoryCategory::Layout, build: stories::modal::build },
    StoryEntry { name: "Popover", category: StoryCategory::Layout, build: stories::popover::build },
];

// ---------------------------------------------------------------------------
// Gallery
// ---------------------------------------------------------------------------

struct Gallery {
    selected_index: usize,
    current_view: Option<AnyView>,
    selected_theme: SharedString,
    _appearance_sub: Option<Subscription>,
    _theme_watcher: Option<engram::theme::hot_reload::ThemeWatcher>,
}

impl Gallery {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let current_view = Some((STORIES[0].build)(window, cx.deref_mut()));
        Self {
            selected_index: 0,
            current_view,
            selected_theme: cx.theme().name.clone(),
            _appearance_sub: None,
            _theme_watcher: None,
        }
    }

    fn select_story(&mut self, index: usize, cx: &mut Context<Self>) {
        if index != self.selected_index && index < STORIES.len() {
            self.selected_index = index;
            self.current_view = None;
            cx.notify();
        }
    }
}

impl Render for Gallery {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Lazily rebuild the story view when the selection changed.
        if self.current_view.is_none() {
            self.current_view =
                Some((STORIES[self.selected_index].build)(window, cx.deref_mut()));
        }
        let view = self.current_view.clone().unwrap();
        let colors = cx.theme().colors();
        let weak = cx.entity().downgrade();

        h_flex()
            .size_full()
            .bg(colors.background)
            // ---- Sidebar ----
            .child(
                v_flex()
                    .w(px(240.0))
                    .h_full()
                    .flex_shrink_0()
                    .border_r_1()
                    .border_color(colors.border)
                    .bg(colors.surface_background)
                    .child(
                        v_flex()
                            .p(Spacing::Medium.pixels())
                            .gap(Spacing::Medium.pixels())
                            .child(
                                Headline::new("engram stories")
                                    .size(HeadlineSize::Small),
                            )
                            .child(self.render_theme_switcher(cx))
                            .child(Divider::horizontal()),
                    )
                    .child(
                        v_flex()
                            .id("sidebar-scroll")
                            .flex_1()
                            .overflow_y_scroll()
                            .p(Spacing::Medium.pixels())
                            .pt_0()
                            .gap(Spacing::Small.pixels())
                            .children(self.render_sidebar_groups(&weak)),
                    ),
            )
            // ---- Content pane ----
            .child(
                v_flex()
                    .id("content-scroll")
                    .flex_1()
                    .h_full()
                    .overflow_y_scroll()
                    .p(Spacing::XXLarge.pixels())
                    .gap(Spacing::Large.pixels())
                    .child(
                        Headline::new(STORIES[self.selected_index].name)
                            .size(HeadlineSize::Medium),
                    )
                    .child(Divider::horizontal())
                    .child(view),
            )
    }
}

impl Gallery {
    fn render_sidebar_groups(
        &self,
        weak: &gpui::WeakEntity<Self>,
    ) -> Vec<impl IntoElement> {
        let mut groups = Vec::new();

        for category in StoryCategory::iter() {
            let entries: Vec<(usize, &StoryEntry)> = STORIES
                .iter()
                .enumerate()
                .filter(|(_, s)| s.category == category)
                .collect();

            if entries.is_empty() {
                continue;
            }

            let mut group = v_flex().gap(Spacing::XSmall.pixels()).child(
                Label::new(category.to_string())
                    .size(LabelSize::XSmall)
                    .color(Color::Muted),
            );

            for (index, entry) in entries {
                let is_selected = index == self.selected_index;
                let weak = weak.clone();
                group = group.child(
                    ListItem::new(SharedString::from(format!("story-{}", entry.name)))
                        .child(Label::new(entry.name))
                        .toggle_state(is_selected)
                        .inset(true)
                        .spacing(ListItemSpacing::Dense)
                        .on_click(move |_event, _window, cx| {
                            weak.update(cx, |this, cx| {
                                this.select_story(index, cx);
                            })
                            .ok();
                        }),
                );
            }

            groups.push(group);
        }
        groups
    }

    fn render_theme_switcher(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme_names = engram::theme::ThemeRegistry::global(cx).names();
        let weak = cx.entity().downgrade();

        h_flex()
            .gap(Spacing::XSmall.pixels())
            .flex_wrap()
            .children(theme_names.into_iter().map(|name| {
                let is_current = name == self.selected_theme;
                let weak = weak.clone();
                let target = name.clone();
                Button::new(
                    SharedString::from(format!("theme-{name}")),
                    name.clone(),
                )
                .size(ButtonSize::Compact)
                .style(if is_current {
                    ButtonStyle::Filled
                } else {
                    ButtonStyle::Subtle
                })
                .toggle_state(is_current)
                .on_click(move |_event, _window, cx| {
                    let target = target.clone();
                    weak.update(cx, |this, cx| {
                        this._appearance_sub = None;
                        if engram::theme::activate_theme(&target, cx).is_ok() {
                            this.selected_theme = target;
                            cx.notify();
                        }
                    })
                    .ok();
                })
            }))
    }
}

// ---------------------------------------------------------------------------
// Theme loading (mirrors showcase.rs)
// ---------------------------------------------------------------------------

fn register_embedded_themes(cx: &mut App) {
    use engram::theme::{Theme, ThemeRegistry};
    use gpui::AssetSource;

    let asset_paths = match Assets.list("themes/") {
        Ok(paths) => paths,
        Err(err) => {
            eprintln!("story: failed to list embedded themes: {err}");
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
                    eprintln!("story: failed to parse {path}: {err}");
                }
            },
            Ok(None) => {}
            Err(err) => {
                eprintln!("story: failed to load {path}: {err}");
            }
        }
    }
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() {
    application().with_assets(Assets).run(|cx: &mut App| {
        engram::theme::init(cx);
        engram::ui::init(cx);
        register_embedded_themes(cx);

        let themes_dir = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../engram-ui/assets/themes"
        );
        let mut theme_watcher =
            match engram::theme::hot_reload::watch_themes_dir(themes_dir, cx) {
                Ok(watcher) => Some(watcher),
                Err(err) => {
                    eprintln!("story: hot reload disabled: {err}");
                    None
                }
            };

        let bounds = Bounds::centered(None, size(px(1100.0), px(760.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| {
                let appearance_sub = engram::theme::sync_with_system_appearance(
                    Default::default(),
                    window,
                    cx,
                );
                let entity: Entity<Gallery> = cx.new(|cx| Gallery::new(window, cx));
                entity.update(cx, |gallery, cx| {
                    gallery.selected_theme = cx.theme().name.clone();
                    gallery._appearance_sub = Some(appearance_sub);
                    gallery._theme_watcher = theme_watcher.take();
                });
                entity
            },
        )
        .unwrap();

        cx.activate(true);
    });
}
