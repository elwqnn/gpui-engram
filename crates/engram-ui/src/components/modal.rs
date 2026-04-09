//! Modal — centered overlay card with a dimmed backdrop.
//!
//! Like [`Popover`](super::popover::Popover), the modal is *stateless*: the
//! parent view holds an `is_open` flag and conditionally inserts a
//! [`modal_overlay`] in its element tree. The overlay covers the full window
//! (via `deferred()`), draws a translucent backdrop, and centers the
//! [`Modal`] content. Clicking the backdrop **or** pressing `Escape`
//! dispatches the caller's `on_dismiss` handler.
//!
//! The parent owns a [`FocusHandle`] for the modal and is responsible for
//! focusing it when the modal opens (so `Escape` has somewhere to land):
//!
//! ```ignore
//! // Construction: create a focus handle once.
//! self.modal_focus = cx.focus_handle();
//!
//! // Opening: flip the flag AND focus the handle.
//! Button::new("open", "Open").on_click(cx.listener(|this, _, window, cx| {
//!     this.modal_open = true;
//!     window.focus(&this.modal_focus, cx);
//!     cx.notify();
//! }))
//!
//! // Rendering: pass a clone of the handle to the overlay helper.
//! .when(self.modal_open, |this| {
//!     this.child(modal_overlay(
//!         self.modal_focus.clone(),
//!         Modal::new().title("Delete file?").child(Label::new("Forever.")),
//!         cx.listener(|this, _, _, cx| { this.modal_open = false; cx.notify(); }),
//!     ))
//! })
//! ```

use std::rc::Rc;

use engram_theme::{ActiveTheme, Radius, Spacing};
use gpui::{
    AnyElement, App, FocusHandle, Hsla, IntoElement, MouseButton, ParentElement, Pixels,
    RenderOnce, SharedString, Window, deferred, div, hsla, prelude::*, px,
};
use smallvec::SmallVec;

use crate::components::label::{Label, LabelCommon, LabelSize};
use crate::components::stack::v_flex;
use crate::styles::ElevationIndex;
use crate::traits::DismissHandler;

fn backdrop() -> Hsla {
    hsla(0.0, 0.0, 0.0, 0.45)
}

/// A centered card with optional title, body children, and footer row.
#[derive(IntoElement)]
pub struct Modal {
    title: Option<SharedString>,
    children: SmallVec<[AnyElement; 4]>,
    footer: Option<AnyElement>,
    width: Pixels,
}

impl Modal {
    pub fn new() -> Self {
        Self {
            title: None,
            children: SmallVec::new(),
            footer: None,
            width: px(420.0),
        }
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn footer(mut self, footer: impl IntoElement) -> Self {
        self.footer = Some(footer.into_any_element());
        self
    }

    pub fn width(mut self, width: Pixels) -> Self {
        self.width = width;
        self
    }
}

impl Default for Modal {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for Modal {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements)
    }
}

impl RenderOnce for Modal {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.theme().colors();
        v_flex()
            .w(self.width)
            .gap(Spacing::Medium.pixels())
            .p(Spacing::Large.pixels())
            .rounded(Radius::Large.pixels())
            .border_1()
            .border_color(colors.border)
            .bg(colors.elevated_surface_background)
            .shadow(ElevationIndex::ModalSurface.shadow(cx))
            .when_some(self.title, |this, title| {
                this.child(Label::new(title).size(LabelSize::Large))
            })
            .child(v_flex().gap(Spacing::Small.pixels()).children(self.children))
            .when_some(self.footer, |this, footer| this.child(footer))
    }
}

/// Wrap a [`Modal`] (or any element) in a full-window backdrop layer that
/// dismisses on **backdrop click** or **`Escape`**.
///
/// # Focus requirement
///
/// The caller passes a [`FocusHandle`] that is expected to be focused *while
/// the overlay is visible*. This is what gives the Escape key somewhere to
/// land — without a focused handle in the overlay's subtree, key events go
/// elsewhere and Esc does nothing.
///
/// The overlay paints inside a [`deferred`] node so it floats above siblings.
/// Clicks inside the modal card are `.occlude()`d so they don't bubble up to
/// the backdrop and cause an accidental dismiss.
pub fn modal_overlay(
    focus_handle: FocusHandle,
    content: impl IntoElement,
    on_dismiss: impl Fn(&mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let on_dismiss: DismissHandler = Rc::new(on_dismiss);
    let click_dismiss = on_dismiss.clone();
    let key_dismiss = on_dismiss;
    deferred(
        div()
            .id("engram-modal-backdrop")
            .track_focus(&focus_handle)
            .absolute()
            .inset_0()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(backdrop())
            // Mouse-down on the backdrop dismisses. We use mouse_down rather
            // than click so the dismiss fires even if the press and release
            // land on different spots of the backdrop.
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
                // Any mouse_down inside the card is swallowed so the
                // backdrop's dismiss handler never sees it.
                div()
                    .occlude()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .child(content),
            ),
    )
    .with_priority(2)
}
