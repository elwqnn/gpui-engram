//! Stepper — a compact numeric increment / decrement control.
//!
//! Two icon buttons (minus / plus) flanking a value display. The parent
//! owns the value and receives changes via `on_change`. Stateless like
//! every other engram component — the stepper just renders the current
//! value and fires callbacks.

use std::rc::Rc;

use engram_theme::{ActiveTheme, Color, Radius, Spacing};
use gpui::{
    App, ElementId, IntoElement, RenderOnce, SharedString, Styled, Window, div, prelude::*, px,
};

use crate::components::icon::{Icon, IconName, IconSize};
use crate::components::label::{Label, LabelCommon, LabelSize};
use crate::components::stack::h_flex;
use crate::traits::Disableable;
use crate::traits::handlers::F64Handler;

/// A compact numeric control with decrement/increment buttons.
#[derive(IntoElement)]
pub struct Stepper {
    id: ElementId,
    value: f64,
    min: f64,
    max: f64,
    step: f64,
    disabled: bool,
    label: Option<SharedString>,
    on_change: Option<F64Handler>,
}

impl Stepper {
    pub fn new(id: impl Into<ElementId>, value: f64) -> Self {
        Self {
            id: id.into(),
            value,
            min: f64::MIN,
            max: f64::MAX,
            step: 1.0,
            disabled: false,
            label: None,
            on_change: None,
        }
    }

    pub fn min(mut self, min: f64) -> Self {
        self.min = min;
        self
    }

    pub fn max(mut self, max: f64) -> Self {
        self.max = max;
        self
    }

    pub fn step(mut self, step: f64) -> Self {
        self.step = step;
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Register a change handler, invoked with the new value when +/− is clicked.
    pub fn on_change(
        mut self,
        handler: impl Fn(f64, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl Disableable for Stepper {
    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl RenderOnce for Stepper {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.theme().colors();
        let label_color = if self.disabled {
            Color::Disabled
        } else {
            Color::Default
        };

        let at_min = self.value <= self.min;
        let at_max = self.value >= self.max;

        let btn_size = px(24.0);
        let icon_color = |suppressed: bool| {
            if self.disabled || suppressed {
                Color::Disabled
            } else {
                Color::Default
            }
        };

        let value_text = if self.step == self.step.round() {
            format!("{}", self.value as i64)
        } else {
            format!("{:.1}", self.value)
        };

        // Decrement button
        let dec_enabled = !self.disabled && !at_min;
        let dec = div()
            .id(SharedString::from(format!("{}-dec", self.id)))
            .size(btn_size)
            .flex()
            .items_center()
            .justify_center()
            .rounded(Radius::Small.pixels())
            .border_1()
            .border_color(colors.border_variant)
            .bg(colors.ghost_element_background)
            .when(dec_enabled, |this| {
                this.cursor_pointer()
                    .hover(|s| s.bg(colors.ghost_element_hover))
            })
            .child(
                Icon::new(IconName::Dash)
                    .size(IconSize::Small)
                    .color(icon_color(at_min)),
            )
            .when_some(
                dec_enabled.then_some(self.on_change.clone()).flatten(),
                |this, handler| {
                    let next = (self.value - self.step).max(self.min);
                    this.on_click(move |_, window, cx| handler(next, window, cx))
                },
            );

        // Increment button
        let inc_enabled = !self.disabled && !at_max;
        let inc = div()
            .id(SharedString::from(format!("{}-inc", self.id)))
            .size(btn_size)
            .flex()
            .items_center()
            .justify_center()
            .rounded(Radius::Small.pixels())
            .border_1()
            .border_color(colors.border_variant)
            .bg(colors.ghost_element_background)
            .when(inc_enabled, |this| {
                this.cursor_pointer()
                    .hover(|s| s.bg(colors.ghost_element_hover))
            })
            .child(
                Icon::new(IconName::Plus)
                    .size(IconSize::Small)
                    .color(icon_color(at_max)),
            )
            .when_some(
                inc_enabled.then_some(self.on_change).flatten(),
                |this, handler| {
                    let next = (self.value + self.step).min(self.max);
                    this.on_click(move |_, window, cx| handler(next, window, cx))
                },
            );

        h_flex()
            .gap(Spacing::Small.pixels())
            .items_center()
            .when_some(self.label, |this, label| {
                this.child(Label::new(label).size(LabelSize::Default).color(label_color))
            })
            .child(
                h_flex()
                    .gap(Spacing::XXSmall.pixels())
                    .items_center()
                    .child(dec)
                    .child(
                        div()
                            .min_w(px(32.0))
                            .flex()
                            .justify_center()
                            .child(Label::new(value_text).size(LabelSize::Default).color(label_color)),
                    )
                    .child(inc),
            )
    }
}
