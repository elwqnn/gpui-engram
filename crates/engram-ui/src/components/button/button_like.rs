//! [`ButtonLike`] — the shared chrome behind every engram button.
//!
//! `ButtonLike` is a thin wrapper around a `gpui::Div` that handles the bits
//! that *every* button needs in lockstep — id, focus tracking, tooltip,
//! click forwarding, hover/active palettes, rounding, optional border. Both
//! [`Button`](super::button::Button) and
//! [`IconButton`](super::icon_button::IconButton) compose this struct
//! internally and forward most of their builder methods through
//! [`ButtonCommon`].
//!
//! `ButtonLike` is also exported on its own so callers who need a
//! freeform "rounded clickable surface with engram's button states" — a
//! card-like trigger, a custom dropdown anchor — can build directly on it
//! without re-implementing all the chrome. Use it sparingly though: every
//! escape hatch is a place engram's visual language can drift.
//!
//! Mirrors zed's `ui::ButtonLike`, scoped down to the bits engram actually
//! exercises. Notably absent: dynamic spacing, focus-visible rings,
//! right-click handling, configurable corner rounding, the `Component`
//! preview registry. Add them when a real consumer needs them.

use std::rc::Rc;

use engram_theme::{ActiveTheme, Radius};
use gpui::{
    AnyElement, AnyView, App, ClickEvent, CursorStyle, DefiniteLength, Div, ElementId, FocusHandle,
    Hsla, IntoElement, ParentElement, Pixels, RenderOnce, StyleRefinement, Window, div, prelude::*,
    relative, transparent_black,
};
use smallvec::SmallVec;

use crate::styles::ElevationIndex;
use crate::traits::{
    ClickHandler, Clickable, Disableable, StyledExt, ToggleState, Toggleable, TooltipBuilder,
};

/// Buttons that can swap their [`ButtonStyle`] when in the selected state.
///
/// Mirrors zed's `SelectableButton`. The trait is intentionally separate
/// from [`Toggleable`] so a selectable-button-with-a-tinted-selected-state
/// can be expressed without forcing every toggleable thing to grow that
/// surface.
pub trait SelectableButton: Toggleable {
    fn selected_style(self, style: ButtonStyle) -> Self;
}

/// The "every button speaks the same dialect" trait — id, style, size,
/// tooltip, elevation layer, tab index, focus tracking.
///
/// Like the rest of engram's behavioural traits in [`crate::traits`], this
/// is **not** used as a generic bound. It exists so every button-like
/// component spells these methods the same way at the call site, and so
/// rustdoc/IDE autocomplete surface them consistently.
pub trait ButtonCommon: Clickable + Disableable {
    /// The button's element id.
    fn id(&self) -> &ElementId;

    /// Set the visual style. Defaults to [`ButtonStyle::Filled`].
    fn style(self, style: ButtonStyle) -> Self;

    /// Set the size preset. Defaults to [`ButtonSize::Default`].
    fn size(self, size: ButtonSize) -> Self;

    /// Attach a tooltip builder. Typically used with
    /// [`Tooltip::text`](crate::components::Tooltip::text).
    fn tooltip(self, tooltip: impl Fn(&mut Window, &mut App) -> AnyView + 'static) -> Self;

    /// Insert this button into the keyboard tab order at `tab_index`.
    fn tab_index(self, tab_index: isize) -> Self;

    /// Tell the button which [`ElevationIndex`] surface it sits on. The
    /// rendered background of [`ButtonStyle::Filled`] / [`ButtonStyle::Outlined`]
    /// is computed from this layer so the button has the right contrast
    /// against its parent surface.
    fn layer(self, layer: ElevationIndex) -> Self;

    /// Track focus on the given handle. The button itself does not own the
    /// handle — it's borrowed from a parent view that wants to programmatically
    /// focus it.
    fn track_focus(self, focus_handle: &FocusHandle) -> Self;
}

/// The visual variant of a button.
///
/// Roughly mirrors zed's `ButtonStyle`, scoped down to the variants engram
/// actually exercises. The previous engram-only `Primary` variant has been
/// folded into `Tinted(TintColor::Accent)`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ButtonStyle {
    /// Solid filled background. The most prominent style; used for the
    /// default/affirmative action.
    #[default]
    Filled,
    /// A semantic-coloured tint (Accent / Error / Warning / Success) — soft
    /// background plus a coloured border. Used for primary CTAs and
    /// destructive confirmations.
    Tinted(TintColor),
    /// Bordered, transparent-until-hover background. Reads as a "secondary"
    /// CTA next to a filled or tinted primary.
    Outlined,
    /// Like [`ButtonStyle::Outlined`] but with a more recessive (variant)
    /// border tone. Used when an outlined look is needed but should not
    /// compete with surrounding chrome.
    OutlinedGhost,
    /// Transparent until hover. Toolbar / inline-action style.
    Subtle,
    /// Fully transparent in every state. Useful for buttons whose text is
    /// the only thing that should ever draw the eye.
    Transparent,
}

/// Tint flavor for [`ButtonStyle::Tinted`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TintColor {
    #[default]
    Accent,
    Error,
    Warning,
    Success,
}

/// Resolved background + border colors for one (style × state) pair.
#[derive(Debug, Clone, Copy)]
pub(super) struct ButtonLikeStyles {
    pub background: Hsla,
    pub border: Hsla,
}

impl TintColor {
    /// The foreground status color for this tint flavor.
    fn foreground(self, cx: &App) -> Hsla {
        let status = &cx.theme().colors().status;
        match self {
            TintColor::Accent => status.info,
            TintColor::Error => status.error,
            TintColor::Warning => status.warning,
            TintColor::Success => status.success,
        }
    }

    fn enabled_styles(self, cx: &App) -> ButtonLikeStyles {
        let fg = self.foreground(cx);
        ButtonLikeStyles {
            background: fg.opacity(0.15),
            border: fg.opacity(0.55),
        }
    }

    fn hovered_styles(self, cx: &App) -> ButtonLikeStyles {
        let fg = self.foreground(cx);
        ButtonLikeStyles {
            background: fg.opacity(0.25),
            border: fg.opacity(0.60),
        }
    }
}

/// Pick the right element background for the surface this button sits on.
///
/// Each [`ElevationIndex`] gets a slightly different "blank canvas" tone so
/// a filled button on a popover doesn't look identical to one on the root
/// background. Falls back to `element_background` when the layer wasn't
/// set, which is the historical engram default.
fn element_bg_for_layer(layer: Option<ElevationIndex>, cx: &App) -> Hsla {
    let colors = cx.theme().colors();
    match layer {
        Some(ElevationIndex::Background) | None => colors.element_background,
        Some(ElevationIndex::Surface) => colors.surface_background,
        Some(ElevationIndex::ElevatedSurface) => colors.elevated_surface_background,
        Some(ElevationIndex::ModalSurface) => colors.background,
    }
}

impl ButtonStyle {
    pub(super) fn enabled(self, layer: Option<ElevationIndex>, cx: &App) -> ButtonLikeStyles {
        let colors = cx.theme().colors();
        match self {
            ButtonStyle::Filled => ButtonLikeStyles {
                background: element_bg_for_layer(layer, cx),
                border: transparent_black(),
            },
            ButtonStyle::Tinted(tint) => tint.enabled_styles(cx),
            ButtonStyle::Outlined => ButtonLikeStyles {
                background: element_bg_for_layer(layer, cx),
                border: colors.border,
            },
            ButtonStyle::OutlinedGhost => ButtonLikeStyles {
                background: transparent_black(),
                border: colors.border_variant,
            },
            ButtonStyle::Subtle => ButtonLikeStyles {
                background: colors.ghost_element_background,
                border: transparent_black(),
            },
            ButtonStyle::Transparent => ButtonLikeStyles {
                background: transparent_black(),
                border: transparent_black(),
            },
        }
    }

    pub(super) fn hovered(self, _layer: Option<ElevationIndex>, cx: &App) -> ButtonLikeStyles {
        let colors = cx.theme().colors();
        match self {
            ButtonStyle::Filled => ButtonLikeStyles {
                background: colors.element_hover,
                border: transparent_black(),
            },
            // Tinted backgrounds are alpha-blended from the status
            // foreground color; hover bumps the alpha to give feedback
            // without an extra darken pass.
            ButtonStyle::Tinted(tint) => tint.hovered_styles(cx),
            ButtonStyle::Outlined => ButtonLikeStyles {
                background: colors.ghost_element_hover,
                border: colors.border,
            },
            ButtonStyle::OutlinedGhost => ButtonLikeStyles {
                background: colors.ghost_element_hover,
                border: colors.border_variant,
            },
            ButtonStyle::Subtle => ButtonLikeStyles {
                background: colors.ghost_element_hover,
                border: transparent_black(),
            },
            ButtonStyle::Transparent => ButtonLikeStyles {
                background: transparent_black(),
                border: transparent_black(),
            },
        }
    }

    pub(super) fn active(self, _layer: Option<ElevationIndex>, cx: &App) -> ButtonLikeStyles {
        let colors = cx.theme().colors();
        match self {
            ButtonStyle::Filled => ButtonLikeStyles {
                background: colors.element_active,
                border: transparent_black(),
            },
            ButtonStyle::Tinted(tint) => {
                let fg = tint.foreground(cx);
                ButtonLikeStyles {
                    background: fg.opacity(0.32),
                    border: fg.opacity(0.65),
                }
            }
            ButtonStyle::Outlined => ButtonLikeStyles {
                background: colors.element_active,
                border: colors.border,
            },
            ButtonStyle::OutlinedGhost => ButtonLikeStyles {
                background: transparent_black(),
                border: colors.border_variant,
            },
            ButtonStyle::Subtle => ButtonLikeStyles {
                background: colors.ghost_element_active,
                border: transparent_black(),
            },
            ButtonStyle::Transparent => ButtonLikeStyles {
                background: transparent_black(),
                border: transparent_black(),
            },
        }
    }

    pub(super) fn disabled_styles(
        self,
        _layer: Option<ElevationIndex>,
        cx: &App,
    ) -> ButtonLikeStyles {
        let colors = cx.theme().colors();
        match self {
            ButtonStyle::Filled | ButtonStyle::Outlined => ButtonLikeStyles {
                background: colors.element_disabled,
                border: colors.border_disabled,
            },
            ButtonStyle::Tinted(_) => ButtonLikeStyles {
                background: colors.element_disabled,
                border: colors.border_disabled,
            },
            ButtonStyle::OutlinedGhost => ButtonLikeStyles {
                background: transparent_black(),
                border: colors.border_disabled,
            },
            ButtonStyle::Subtle => ButtonLikeStyles {
                background: colors.ghost_element_disabled,
                border: transparent_black(),
            },
            ButtonStyle::Transparent => ButtonLikeStyles {
                background: transparent_black(),
                border: transparent_black(),
            },
        }
    }

    /// Whether this style ever paints a visible border. Drives the
    /// `border_1()` call in render so non-bordered styles don't reserve a
    /// pixel of border space.
    pub(super) fn is_outlined(self) -> bool {
        matches!(self, ButtonStyle::Outlined | ButtonStyle::OutlinedGhost)
    }
}

/// Button height presets. These also drive the inner padding for [`Button`]
/// and [`IconButton`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ButtonSize {
    Compact,
    #[default]
    Default,
    Large,
}

/// Per-corner rounding control for buttons in segmented groups.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) struct ButtonLikeRounding {
    pub top_left: bool,
    pub top_right: bool,
    pub bottom_right: bool,
    pub bottom_left: bool,
}

impl ButtonLikeRounding {
    pub const ALL: Self = Self {
        top_left: true,
        top_right: true,
        bottom_right: true,
        bottom_left: true,
    };
}

/// Shared chrome behind every engram button. See the module docs.
#[derive(IntoElement)]
#[must_use = "ButtonLike does nothing unless rendered"]
pub struct ButtonLike {
    pub(super) base: Div,
    pub(super) id: ElementId,
    pub(super) style: ButtonStyle,
    pub(super) size: ButtonSize,
    pub(super) disabled: bool,
    pub(super) selected: bool,
    pub(super) selected_style: Option<ButtonStyle>,
    pub(super) layer: Option<ElevationIndex>,
    pub(super) focus_handle: Option<FocusHandle>,
    pub(super) tab_index: Option<isize>,
    pub(super) cursor_style: CursorStyle,
    pub(super) tooltip: Option<TooltipBuilder>,
    pub(super) on_click: Option<ClickHandler>,
    pub(super) children: SmallVec<[AnyElement; 2]>,
    pub(super) horizontal_padding: Option<Pixels>,
    pub(super) vertical_padding: Option<Pixels>,
    pub(super) rounding: Option<ButtonLikeRounding>,
    pub(super) width: Option<DefiniteLength>,
}

impl ButtonLike {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            base: div(),
            id: id.into(),
            style: ButtonStyle::default(),
            size: ButtonSize::default(),
            disabled: false,
            selected: false,
            selected_style: None,
            layer: None,
            focus_handle: None,
            tab_index: None,
            cursor_style: CursorStyle::PointingHand,
            tooltip: None,
            on_click: None,
            children: SmallVec::new(),
            horizontal_padding: None,
            vertical_padding: None,
            rounding: Some(ButtonLikeRounding::ALL),
            width: None,
        }
    }

    /// Set per-corner rounding. `None` means no rounding at all.
    pub(crate) fn rounding(mut self, rounding: impl Into<Option<ButtonLikeRounding>>) -> Self {
        self.rounding = rounding.into();
        self
    }

    /// Set a fixed width for this button.
    pub fn width(mut self, width: impl Into<DefiniteLength>) -> Self {
        self.width = Some(width.into());
        self
    }

    /// Set the button to fill its parent width.
    pub fn full_width(mut self) -> Self {
        self.width = Some(relative(1.));
        self
    }

    /// Set the inner padding (horizontal, vertical) of this button. Used by
    /// the wrapping [`Button`](super::button::Button) and
    /// [`IconButton`](super::icon_button::IconButton) to apply their own
    /// size-derived padding through ButtonLike's chrome — the padding has
    /// to live on the same div that paints the background, otherwise the
    /// background hugs the inner content with no breathing room.
    pub fn padding(mut self, horizontal: Pixels, vertical: Pixels) -> Self {
        self.horizontal_padding = Some(horizontal);
        self.vertical_padding = Some(vertical);
        self
    }
}

impl ParentElement for ButtonLike {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl Disableable for ButtonLike {
    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Toggleable for ButtonLike {
    fn toggle_state(mut self, state: impl Into<ToggleState>) -> Self {
        self.selected = state.into().selected();
        self
    }
}

impl SelectableButton for ButtonLike {
    fn selected_style(mut self, style: ButtonStyle) -> Self {
        self.selected_style = Some(style);
        self
    }
}

impl Clickable for ButtonLike {
    fn on_click(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    fn cursor_style(mut self, cursor_style: CursorStyle) -> Self {
        self.cursor_style = cursor_style;
        self
    }
}

impl ButtonCommon for ButtonLike {
    fn id(&self) -> &ElementId {
        &self.id
    }

    fn style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }

    fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    fn tooltip(mut self, tooltip: impl Fn(&mut Window, &mut App) -> AnyView + 'static) -> Self {
        self.tooltip = Some(Rc::new(tooltip));
        self
    }

    fn tab_index(mut self, tab_index: isize) -> Self {
        self.tab_index = Some(tab_index);
        self
    }

    fn layer(mut self, layer: ElevationIndex) -> Self {
        self.layer = Some(layer);
        self
    }

    fn track_focus(mut self, focus_handle: &FocusHandle) -> Self {
        self.focus_handle = Some(focus_handle.clone());
        self
    }
}

impl RenderOnce for ButtonLike {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // When `selected` is true but no explicit `selected_style` was set,
        // fall back to a tinted-accent palette so selection is visible even
        // on low-contrast base styles (Subtle, Transparent).
        let style = if self.selected {
            self.selected_style
                .unwrap_or(ButtonStyle::Tinted(TintColor::Accent))
        } else {
            self.style
        };

        let enabled = style.enabled(self.layer, cx);
        let hovered = style.hovered(self.layer, cx);
        let active = style.active(self.layer, cx);
        let disabled_palette = style.disabled_styles(self.layer, cx);
        let is_outlined = style.is_outlined();
        let is_disabled = self.disabled;
        let cursor = self.cursor_style;

        let on_click = self.on_click;
        let tooltip = self.tooltip;
        let focus_handle = self.focus_handle;
        let tab_index = self.tab_index;
        let children = self.children;
        let horizontal_padding = self.horizontal_padding;
        let vertical_padding = self.vertical_padding;

        self.base
            .id(self.id)
            .h_flex()
            .when_some(self.width, |this, w| this.w(w).justify_center())
            .map(|this| {
                let r = Radius::Medium.pixels();
                match self.rounding {
                    Some(rounding) => this
                        .when(rounding.top_left, |e| e.rounded_tl(r))
                        .when(rounding.top_right, |e| e.rounded_tr(r))
                        .when(rounding.bottom_right, |e| e.rounded_br(r))
                        .when(rounding.bottom_left, |e| e.rounded_bl(r)),
                    None => this,
                }
            })
            .when_some(horizontal_padding, |this, p| this.px(p))
            .when_some(vertical_padding, |this, p| this.py(p))
            .when(is_outlined, |this| this.border_1())
            .map(|this| {
                if is_disabled {
                    let this = this.bg(disabled_palette.background);
                    if is_outlined {
                        this.border_color(disabled_palette.border)
                    } else {
                        this
                    }
                } else {
                    let this = this.bg(enabled.background);
                    let this = if is_outlined {
                        this.border_color(enabled.border)
                    } else {
                        this
                    };
                    this.cursor(cursor)
                        .hover(move |s: StyleRefinement| {
                            let s = s.bg(hovered.background);
                            if is_outlined {
                                s.border_color(hovered.border)
                            } else {
                                s
                            }
                        })
                        .active(move |s: StyleRefinement| {
                            let s = s.bg(active.background);
                            if is_outlined {
                                s.border_color(active.border)
                            } else {
                                s
                            }
                        })
                }
            })
            .when_some(tab_index, |this, ix| this.tab_index(ix))
            .when_some(focus_handle, |this, fh| this.track_focus(&fh))
            .when_some(tooltip, |this, builder| {
                this.tooltip(move |window, cx| builder(window, cx))
            })
            .when_some(on_click.filter(|_| !is_disabled), |this, handler| {
                this.on_click(move |event, window, cx| handler(event, window, cx))
            })
            .children(children)
    }
}
