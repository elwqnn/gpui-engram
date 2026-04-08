//! [`IconButton`] — square icon-only button. Composes [`ButtonLike`].
//!
//! Same shape as [`Button`](super::button::Button) but renders only an
//! [`Icon`] and uses smaller, square padding so the hit target stays
//! square. The label-color logic is identical (Disabled / Selected /
//! Default).

use engram_theme::Color;
use gpui::{
    AnyView, App, ClickEvent, CursorStyle, ElementId, FocusHandle, IntoElement, ParentElement,
    Pixels, RenderOnce, Window, px,
};

use crate::components::button::button_like::{
    ButtonCommon, ButtonLike, ButtonSize, ButtonStyle, SelectableButton,
};
use crate::components::icon::{Icon, IconName, IconSize};
use crate::styles::ElevationIndex;
use crate::traits::{Clickable, Disableable, ToggleState, Toggleable};

/// A square icon-only button.
#[derive(IntoElement)]
pub struct IconButton {
    base: ButtonLike,
    icon: IconName,
}

impl IconButton {
    pub fn new(id: impl Into<ElementId>, icon: IconName) -> Self {
        Self {
            base: ButtonLike::new(id),
            icon,
        }
    }
}

impl Disableable for IconButton {
    fn disabled(mut self, disabled: bool) -> Self {
        self.base = self.base.disabled(disabled);
        self
    }
}

impl Toggleable for IconButton {
    fn toggle_state(mut self, state: impl Into<ToggleState>) -> Self {
        self.base = self.base.toggle_state(state);
        self
    }
}

impl SelectableButton for IconButton {
    fn selected_style(mut self, style: ButtonStyle) -> Self {
        self.base = self.base.selected_style(style);
        self
    }
}

impl Clickable for IconButton {
    fn on_click(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.base = self.base.on_click(handler);
        self
    }

    fn cursor_style(mut self, cursor_style: CursorStyle) -> Self {
        self.base = self.base.cursor_style(cursor_style);
        self
    }
}

impl ButtonCommon for IconButton {
    fn id(&self) -> &ElementId {
        self.base.id()
    }

    fn style(mut self, style: ButtonStyle) -> Self {
        self.base = self.base.style(style);
        self
    }

    fn size(mut self, size: ButtonSize) -> Self {
        self.base = self.base.size(size);
        self
    }

    fn tooltip(mut self, tooltip: impl Fn(&mut Window, &mut App) -> AnyView + 'static) -> Self {
        self.base = self.base.tooltip(tooltip);
        self
    }

    fn tab_index(mut self, tab_index: isize) -> Self {
        self.base = self.base.tab_index(tab_index);
        self
    }

    fn layer(mut self, layer: ElevationIndex) -> Self {
        self.base = self.base.layer(layer);
        self
    }

    fn track_focus(mut self, focus_handle: &FocusHandle) -> Self {
        self.base = self.base.track_focus(focus_handle);
        self
    }
}

impl RenderOnce for IconButton {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let is_disabled = self.base.disabled;
        let is_selected = self.base.selected;
        let size = self.base.size;
        let pad = padding_for(size);
        let icon_size = icon_size_for(size);

        let icon_color = if is_disabled {
            Color::Disabled
        } else if is_selected {
            Color::Selected
        } else {
            Color::Default
        };

        self.base
            .padding(pad, pad)
            .child(Icon::new(self.icon).size(icon_size).color(icon_color))
    }
}

/// Square padding step. Engram keeps the icon-button hit target tighter
/// than [`Button`]'s rectangular padding so toolbar densities still feel
/// reasonable at `ButtonSize::Default`.
fn padding_for(size: ButtonSize) -> Pixels {
    match size {
        ButtonSize::Compact => px(4.0),
        ButtonSize::Default => px(6.0),
        ButtonSize::Large => px(8.0),
    }
}

fn icon_size_for(size: ButtonSize) -> IconSize {
    match size {
        ButtonSize::Compact => IconSize::Small,
        ButtonSize::Default => IconSize::Medium,
        ButtonSize::Large => IconSize::Large,
    }
}
