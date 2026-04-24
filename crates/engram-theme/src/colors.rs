//! Semantic color tokens used by every engram component.

use gpui::Hsla;

/// A semantic color reference that components use instead of raw [`Hsla`].
///
/// At render time, [`Color::hsla`] resolves against the active [`Theme`](crate::Theme).
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Color {
    /// Default text / foreground color.
    #[default]
    Default,
    /// Muted / secondary foreground.
    Muted,
    /// Placeholder foreground (e.g. empty input hint).
    Placeholder,
    /// Disabled foreground.
    Disabled,
    /// Accent foreground (links, emphasis).
    Accent,
    /// Selected foreground - used for items in the selected/active state.
    Selected,
    /// Hint or suggestion text. Typically a soft blue.
    Hint,
    /// Visually hidden / strongly de-emphasized foreground.
    Hidden,
    /// Intentionally ignored item (e.g. a gitignored file in a tree view).
    Ignored,
    /// Success / positive status.
    Success,
    /// Warning / caution status.
    Warning,
    /// Error / destructive status.
    Error,
    /// Informational status.
    Info,
    /// Raw color, bypassing the theme.
    Custom(Hsla),
}

impl Color {
    /// Resolve this semantic color to an [`Hsla`] using the given [`ThemeColors`].
    pub fn hsla(&self, colors: &ThemeColors) -> Hsla {
        match self {
            Self::Default => colors.text,
            Self::Muted => colors.text_muted,
            Self::Placeholder => colors.text_placeholder,
            Self::Disabled => colors.text_disabled,
            Self::Accent => colors.text_accent,
            Self::Selected => colors.text_accent,
            Self::Hint => colors.status.hint,
            Self::Hidden => colors.status.hidden,
            Self::Ignored => colors.status.ignored,
            Self::Success => colors.status.success,
            Self::Warning => colors.status.warning,
            Self::Error => colors.status.error,
            Self::Info => colors.status.info,
            Self::Custom(hsla) => *hsla,
        }
    }
}

/// Status color group: the diagnostic / informational flavors, each with a
/// base foreground variant plus a background and border for status surfaces
/// (e.g. inline banners, toast chrome, gutter ranges).
///
/// Mirrors zed's `theme::StatusColors`, scoped down to the seven flavors
/// engram actually uses today. Add more (`conflict`, `created`, etc.) only
/// when a real component needs them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StatusColors {
    pub info: Hsla,
    pub info_background: Hsla,
    pub info_border: Hsla,

    pub success: Hsla,
    pub success_background: Hsla,
    pub success_border: Hsla,

    pub warning: Hsla,
    pub warning_background: Hsla,
    pub warning_border: Hsla,

    pub error: Hsla,
    pub error_background: Hsla,
    pub error_border: Hsla,

    /// Hint or suggestion text. Typically a soft blue.
    pub hint: Hsla,
    pub hint_background: Hsla,
    pub hint_border: Hsla,

    /// Strongly de-emphasized - items that are present but should not draw
    /// the eye (e.g. a hidden file in a tree).
    pub hidden: Hsla,
    pub hidden_background: Hsla,
    pub hidden_border: Hsla,

    /// Items intentionally ignored (e.g. gitignored entries).
    pub ignored: Hsla,
    pub ignored_background: Hsla,
    pub ignored_border: Hsla,
}

/// The slim semantic color palette powering every engram component.
///
/// This is deliberately much smaller than Zed's `ThemeColors` (~40 tokens vs.
/// ~150). We add tokens only when a real component needs them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThemeColors {
    // Surfaces
    /// Window / root background.
    pub background: Hsla,
    /// Grounded surface (panel, sidebar).
    pub surface_background: Hsla,
    /// Elevated surface (popover, dropdown, card).
    pub elevated_surface_background: Hsla,

    // Borders
    /// Default border color.
    pub border: Hsla,
    /// Subtle border used for dividers between related content.
    pub border_variant: Hsla,
    /// Border for keyboard focus ring.
    pub border_focused: Hsla,
    /// Border for the active / selected state.
    pub border_selected: Hsla,
    /// Border for disabled elements.
    pub border_disabled: Hsla,
    /// A fully transparent border. Used as a placeholder so that elements
    /// which gain a border on state change don't reflow.
    pub border_transparent: Hsla,

    // Foreground text
    pub text: Hsla,
    pub text_muted: Hsla,
    pub text_placeholder: Hsla,
    pub text_disabled: Hsla,
    pub text_accent: Hsla,

    // Foreground icons
    pub icon: Hsla,
    pub icon_muted: Hsla,
    pub icon_disabled: Hsla,
    pub icon_accent: Hsla,

    // Filled (opaque) interactive element backgrounds
    pub element_background: Hsla,
    pub element_hover: Hsla,
    pub element_active: Hsla,
    pub element_selected: Hsla,
    pub element_disabled: Hsla,

    // Ghost (transparent) interactive element backgrounds
    /// Resting background for a ghost element. Almost always fully
    /// transparent - kept as a token so the layering reads consistently with
    /// `element_background`.
    pub ghost_element_background: Hsla,
    pub ghost_element_hover: Hsla,
    pub ghost_element_active: Hsla,
    pub ghost_element_selected: Hsla,
    pub ghost_element_disabled: Hsla,

    // Status / semantic
    pub status: StatusColors,

    // Accent (primary brand color)
    pub accent: Hsla,
}
