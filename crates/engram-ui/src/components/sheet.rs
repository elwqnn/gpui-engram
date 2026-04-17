//! Sheet — a panel overlay that slides in from a window edge.
//!
//! Follows the same pattern as [`Modal`](super::modal): the parent owns the
//! open state and conditionally inserts a [`sheet_overlay`] in the element
//! tree. The overlay draws a full-window backdrop (click or Escape to
//! dismiss) and positions the sheet content along one edge.
//!
//! ```ignore
//! .when(self.sheet_open, |this| {
//!     this.child(sheet_overlay(
//!         self.sheet_focus.clone(),
//!         Sheet::new().title("Details").side(SheetSide::Right)
//!             .child(Label::new("Panel content")),
//!         cx.listener(|this, _, _, cx| { this.sheet_open = false; cx.notify(); }),
//!     ))
//! })
//! ```

use std::rc::Rc;

use engram_theme::{ActiveTheme, Spacing};
use gpui::{
    AnyElement, App, FocusHandle, IntoElement, MouseButton, ParentElement, Pixels, RenderOnce,
    SharedString, Window, deferred, div, hsla, prelude::*, px,
};
use smallvec::SmallVec;

use crate::components::label::{Label, LabelCommon, LabelSize};
use crate::components::stack::v_flex;
use crate::styles::ElevationIndex;
use crate::traits::DismissHandler;

/// Which window edge the sheet attaches to.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SheetSide {
    #[default]
    Right,
    Left,
    Bottom,
}

/// A panel overlay surface.
#[derive(IntoElement)]
#[must_use = "Sheet does nothing unless rendered"]
pub struct Sheet {
    side: SheetSide,
    title: Option<SharedString>,
    children: SmallVec<[AnyElement; 4]>,
    width: Pixels,
    /// Only used for `SheetSide::Bottom`.
    height: Pixels,
}

impl Sheet {
    pub fn new() -> Self {
        Self {
            side: SheetSide::default(),
            title: None,
            children: SmallVec::new(),
            width: px(360.0),
            height: px(300.0),
        }
    }

    pub fn side(mut self, side: SheetSide) -> Self {
        self.side = side;
        self
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the panel width (used for Left/Right sides).
    pub fn width(mut self, width: Pixels) -> Self {
        self.width = width;
        self
    }

    /// Set the panel height (used for Bottom side).
    pub fn height(mut self, height: Pixels) -> Self {
        self.height = height;
        self
    }
}

impl Default for Sheet {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for Sheet {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements)
    }
}

impl RenderOnce for Sheet {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.theme().colors();

        let panel = v_flex()
            .gap(Spacing::Medium.pixels())
            .p(Spacing::Large.pixels())
            .bg(colors.elevated_surface_background)
            .border_1()
            .border_color(colors.border)
            .shadow(ElevationIndex::ModalSurface.shadow(cx))
            .when_some(self.title, |this, title| {
                this.child(Label::new(title).size(LabelSize::Large))
            })
            .child(
                v_flex()
                    .gap(Spacing::Small.pixels())
                    .children(self.children),
            );

        // Size the panel based on the side it attaches to.
        match self.side {
            SheetSide::Right | SheetSide::Left => panel.w(self.width).h_full(),
            SheetSide::Bottom => panel.w_full().h(self.height),
        }
    }
}

/// Wrap a [`Sheet`] in a full-window backdrop layer, similar to
/// [`modal_overlay`](super::modal::modal_overlay).
pub fn sheet_overlay(
    focus_handle: FocusHandle,
    sheet: Sheet,
    on_dismiss: impl Fn(&mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let on_dismiss: DismissHandler = Rc::new(on_dismiss);
    let click_dismiss = on_dismiss.clone();
    let key_dismiss = on_dismiss;
    let side = sheet.side;

    deferred(
        div()
            .id("engram-sheet-backdrop")
            .track_focus(&focus_handle)
            .absolute()
            .inset_0()
            .size_full()
            .flex()
            // Position the panel based on the side.
            .when(side == SheetSide::Right, |this| {
                this.flex_row().justify_end().items_stretch()
            })
            .when(side == SheetSide::Left, |this| {
                this.flex_row().justify_start().items_stretch()
            })
            .when(side == SheetSide::Bottom, |this| {
                this.flex_col().justify_end().items_stretch()
            })
            .bg(hsla(0.0, 0.0, 0.0, 0.35))
            .on_mouse_down(MouseButton::Left, move |_event, window, cx| {
                click_dismiss(window, cx);
            })
            .on_key_down(move |event, window, cx| {
                if event.keystroke.key == "escape" {
                    key_dismiss(window, cx);
                    cx.stop_propagation();
                }
            })
            .child(
                div()
                    .occlude()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .child(sheet),
            ),
    )
    .with_priority(2)
}
