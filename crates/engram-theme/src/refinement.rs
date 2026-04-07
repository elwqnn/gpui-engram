//! Partial overrides on top of a base [`ThemeColors`].
//!
//! A [`ThemeColorsRefinement`] is "what a user theme file is allowed to set":
//! every field mirrors [`ThemeColors`] but wrapped in [`Option`]. When loading
//! JSON, fields the user omits stay `None` and fall through to whatever base
//! theme the refinement is applied on top of.
//!
//! This is the hand-rolled counterpart to zed's `#[derive(Refineable)]`
//! proc-macro — we keep the refinement shape in lock-step with
//! [`ThemeColors`] manually, so adding a color token means adding it in both
//! [`colors.rs`](crate::colors) and here.

use gpui::Hsla;
use serde::{Deserialize, Serialize};

use crate::colors::{StatusColors, ThemeColors};

/// A partial [`StatusColors`]. All fields optional.
#[derive(Debug, Clone, Copy, Default, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct StatusColorsRefinement {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<Hsla>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info_background: Option<Hsla>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info_border: Option<Hsla>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub success: Option<Hsla>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success_background: Option<Hsla>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success_border: Option<Hsla>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<Hsla>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning_background: Option<Hsla>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning_border: Option<Hsla>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Hsla>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_background: Option<Hsla>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_border: Option<Hsla>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<Hsla>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint_background: Option<Hsla>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint_border: Option<Hsla>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden: Option<Hsla>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden_background: Option<Hsla>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden_border: Option<Hsla>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignored: Option<Hsla>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignored_background: Option<Hsla>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignored_border: Option<Hsla>,
}

impl StatusColorsRefinement {
    /// Copy every `Some(..)` field of `self` onto `base`, leaving the rest
    /// of `base` untouched.
    pub fn refine(self, base: &mut StatusColors) {
        if let Some(v) = self.info {
            base.info = v;
        }
        if let Some(v) = self.info_background {
            base.info_background = v;
        }
        if let Some(v) = self.info_border {
            base.info_border = v;
        }

        if let Some(v) = self.success {
            base.success = v;
        }
        if let Some(v) = self.success_background {
            base.success_background = v;
        }
        if let Some(v) = self.success_border {
            base.success_border = v;
        }

        if let Some(v) = self.warning {
            base.warning = v;
        }
        if let Some(v) = self.warning_background {
            base.warning_background = v;
        }
        if let Some(v) = self.warning_border {
            base.warning_border = v;
        }

        if let Some(v) = self.error {
            base.error = v;
        }
        if let Some(v) = self.error_background {
            base.error_background = v;
        }
        if let Some(v) = self.error_border {
            base.error_border = v;
        }

        if let Some(v) = self.hint {
            base.hint = v;
        }
        if let Some(v) = self.hint_background {
            base.hint_background = v;
        }
        if let Some(v) = self.hint_border {
            base.hint_border = v;
        }

        if let Some(v) = self.hidden {
            base.hidden = v;
        }
        if let Some(v) = self.hidden_background {
            base.hidden_background = v;
        }
        if let Some(v) = self.hidden_border {
            base.hidden_border = v;
        }

        if let Some(v) = self.ignored {
            base.ignored = v;
        }
        if let Some(v) = self.ignored_background {
            base.ignored_background = v;
        }
        if let Some(v) = self.ignored_border {
            base.ignored_border = v;
        }
    }

    /// Wrap every field of `colors` in `Some`. Useful for dumping a base
    /// theme to JSON — nothing is skipped on serialize.
    pub fn from_full(colors: &StatusColors) -> Self {
        Self {
            info: Some(colors.info),
            info_background: Some(colors.info_background),
            info_border: Some(colors.info_border),

            success: Some(colors.success),
            success_background: Some(colors.success_background),
            success_border: Some(colors.success_border),

            warning: Some(colors.warning),
            warning_background: Some(colors.warning_background),
            warning_border: Some(colors.warning_border),

            error: Some(colors.error),
            error_background: Some(colors.error_background),
            error_border: Some(colors.error_border),

            hint: Some(colors.hint),
            hint_background: Some(colors.hint_background),
            hint_border: Some(colors.hint_border),

            hidden: Some(colors.hidden),
            hidden_background: Some(colors.hidden_background),
            hidden_border: Some(colors.hidden_border),

            ignored: Some(colors.ignored),
            ignored_background: Some(colors.ignored_background),
            ignored_border: Some(colors.ignored_border),
        }
    }

    /// True when no field is overridden. Used so parent refinements can
    /// skip the empty `"status": {}` object on serialize.
    pub(crate) fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

/// A partial [`ThemeColors`]. All fields optional.
#[derive(Debug, Clone, Copy, Default, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ThemeColorsRefinement {
    // Surfaces ---------------------------------------------------------------
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<Hsla>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub surface_background: Option<Hsla>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elevated_surface_background: Option<Hsla>,

    // Borders ----------------------------------------------------------------
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border: Option<Hsla>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_variant: Option<Hsla>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_focused: Option<Hsla>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_selected: Option<Hsla>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_disabled: Option<Hsla>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_transparent: Option<Hsla>,

    // Foreground text --------------------------------------------------------
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<Hsla>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_muted: Option<Hsla>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_placeholder: Option<Hsla>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_disabled: Option<Hsla>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_accent: Option<Hsla>,

    // Foreground icons -------------------------------------------------------
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<Hsla>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_muted: Option<Hsla>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_disabled: Option<Hsla>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_accent: Option<Hsla>,

    // Filled (opaque) interactive element backgrounds ------------------------
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element_background: Option<Hsla>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element_hover: Option<Hsla>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element_active: Option<Hsla>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element_selected: Option<Hsla>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element_disabled: Option<Hsla>,

    // Ghost (transparent) interactive element backgrounds --------------------
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ghost_element_background: Option<Hsla>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ghost_element_hover: Option<Hsla>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ghost_element_active: Option<Hsla>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ghost_element_selected: Option<Hsla>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ghost_element_disabled: Option<Hsla>,

    // Status / semantic ------------------------------------------------------
    #[serde(default, skip_serializing_if = "StatusColorsRefinement::is_empty")]
    pub status: StatusColorsRefinement,

    // Accent (primary brand color) -------------------------------------------
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accent: Option<Hsla>,
}

impl ThemeColorsRefinement {
    /// Copy every `Some(..)` field of `self` onto `base`, leaving the rest
    /// of `base` untouched. Nested [`StatusColorsRefinement`] is recursed
    /// into the same way.
    pub fn refine(self, base: &mut ThemeColors) {
        if let Some(v) = self.background {
            base.background = v;
        }
        if let Some(v) = self.surface_background {
            base.surface_background = v;
        }
        if let Some(v) = self.elevated_surface_background {
            base.elevated_surface_background = v;
        }

        if let Some(v) = self.border {
            base.border = v;
        }
        if let Some(v) = self.border_variant {
            base.border_variant = v;
        }
        if let Some(v) = self.border_focused {
            base.border_focused = v;
        }
        if let Some(v) = self.border_selected {
            base.border_selected = v;
        }
        if let Some(v) = self.border_disabled {
            base.border_disabled = v;
        }
        if let Some(v) = self.border_transparent {
            base.border_transparent = v;
        }

        if let Some(v) = self.text {
            base.text = v;
        }
        if let Some(v) = self.text_muted {
            base.text_muted = v;
        }
        if let Some(v) = self.text_placeholder {
            base.text_placeholder = v;
        }
        if let Some(v) = self.text_disabled {
            base.text_disabled = v;
        }
        if let Some(v) = self.text_accent {
            base.text_accent = v;
        }

        if let Some(v) = self.icon {
            base.icon = v;
        }
        if let Some(v) = self.icon_muted {
            base.icon_muted = v;
        }
        if let Some(v) = self.icon_disabled {
            base.icon_disabled = v;
        }
        if let Some(v) = self.icon_accent {
            base.icon_accent = v;
        }

        if let Some(v) = self.element_background {
            base.element_background = v;
        }
        if let Some(v) = self.element_hover {
            base.element_hover = v;
        }
        if let Some(v) = self.element_active {
            base.element_active = v;
        }
        if let Some(v) = self.element_selected {
            base.element_selected = v;
        }
        if let Some(v) = self.element_disabled {
            base.element_disabled = v;
        }

        if let Some(v) = self.ghost_element_background {
            base.ghost_element_background = v;
        }
        if let Some(v) = self.ghost_element_hover {
            base.ghost_element_hover = v;
        }
        if let Some(v) = self.ghost_element_active {
            base.ghost_element_active = v;
        }
        if let Some(v) = self.ghost_element_selected {
            base.ghost_element_selected = v;
        }
        if let Some(v) = self.ghost_element_disabled {
            base.ghost_element_disabled = v;
        }

        self.status.refine(&mut base.status);

        if let Some(v) = self.accent {
            base.accent = v;
        }
    }

    /// Wrap every field of `colors` in `Some`. Useful for dumping a base
    /// theme to JSON — nothing is skipped on serialize.
    pub fn from_full(colors: &ThemeColors) -> Self {
        Self {
            background: Some(colors.background),
            surface_background: Some(colors.surface_background),
            elevated_surface_background: Some(colors.elevated_surface_background),

            border: Some(colors.border),
            border_variant: Some(colors.border_variant),
            border_focused: Some(colors.border_focused),
            border_selected: Some(colors.border_selected),
            border_disabled: Some(colors.border_disabled),
            border_transparent: Some(colors.border_transparent),

            text: Some(colors.text),
            text_muted: Some(colors.text_muted),
            text_placeholder: Some(colors.text_placeholder),
            text_disabled: Some(colors.text_disabled),
            text_accent: Some(colors.text_accent),

            icon: Some(colors.icon),
            icon_muted: Some(colors.icon_muted),
            icon_disabled: Some(colors.icon_disabled),
            icon_accent: Some(colors.icon_accent),

            element_background: Some(colors.element_background),
            element_hover: Some(colors.element_hover),
            element_active: Some(colors.element_active),
            element_selected: Some(colors.element_selected),
            element_disabled: Some(colors.element_disabled),

            ghost_element_background: Some(colors.ghost_element_background),
            ghost_element_hover: Some(colors.ghost_element_hover),
            ghost_element_active: Some(colors.ghost_element_active),
            ghost_element_selected: Some(colors.ghost_element_selected),
            ghost_element_disabled: Some(colors.ghost_element_disabled),

            status: StatusColorsRefinement::from_full(&colors.status),

            accent: Some(colors.accent),
        }
    }
}
