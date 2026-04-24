//! Hand-tuned default themes.
//!
//! The defaults mirror the engram design draft ([`assets/engram.html`]):
//! warm-tinted neutral surfaces paired with an amber accent, authored
//! directly in OKLCH via the [`oklch`] helper so each value reads exactly
//! like the CSS draft it came from. Conversion is Björn Ottosson's
//! OKLab → linear-sRGB, then the sRGB gamma encode.
//!
//! Themes that want a different vibe (gruvbox, solarized, a pure
//! achromatic variant) override the defaults via JSON or by building a
//! fresh `Theme`.

use gpui::{Hsla, SharedString};

use crate::color_string::{oklch, oklch_to_hsla};
use crate::colors::{StatusColors, ThemeColors};
use crate::{Appearance, Theme};

/// Transparent black. Used for `ghost_element_background` and
/// `border_transparent` - cheaper than constructing via oklch.
const TRANSPARENT: Hsla = Hsla {
    h: 0.0,
    s: 0.0,
    l: 0.0,
    a: 0.0,
};

/// The default dark theme - warm-tinted surfaces with an amber accent,
/// tuned from the engram design draft.
pub fn dark() -> Theme {
    // --bg / --surface / --surface-2
    let background = oklch(0.18, 0.004, 70.0);
    let surface = oklch(0.21, 0.004, 70.0);
    let surface_2 = oklch(0.24, 0.004, 70.0);

    // Borders: default + strong.
    let border = oklch(0.28, 0.004, 70.0);
    let border_strong = oklch(0.36, 0.005, 70.0);

    // Foreground ramp.
    let fg = oklch(0.94, 0.004, 85.0);
    let fg_2 = oklch(0.78, 0.005, 85.0);
    let fg_muted = oklch(0.58, 0.005, 80.0);
    let fg_dim = oklch(0.42, 0.004, 75.0);

    // Interactive element ramp (filled).
    let hover = oklch(0.26, 0.004, 70.0);
    let active = oklch(0.30, 0.004, 70.0);

    // Mono accent (near-fg) per draft's `data-accent="mono"` dark variant.
    let accent = oklch(0.94, 0.004, 85.0);
    let accent_bg = oklch(0.32, 0.004, 70.0);

    // Status foregrounds from the draft; `_background` / `_border` keep
    // engram's alpha-tint formula so severity surfaces stay readable.
    let info = oklch(0.72, 0.09, 235.0);
    let success = oklch(0.72, 0.13, 150.0);
    let warning = oklch(0.78, 0.14, 82.0);
    let error = oklch(0.68, 0.17, 25.0);

    // Neutrals for hidden / ignored slots (not in the draft).
    let hidden = oklch(0.52, 0.003, 75.0);
    let ignored = oklch(0.44, 0.003, 75.0);

    Theme {
        name: SharedString::new_static("Engram Dark"),
        appearance: Appearance::Dark,
        colors: ThemeColors {
            background,
            surface_background: surface,
            elevated_surface_background: surface_2,

            border,
            border_variant: surface,
            border_focused: accent,
            border_selected: border_strong,
            border_disabled: surface,
            border_transparent: TRANSPARENT,

            text: fg,
            text_muted: fg_2,
            text_placeholder: fg_muted,
            text_disabled: fg_dim,
            text_accent: accent,

            icon: fg,
            icon_muted: fg_muted,
            icon_disabled: fg_dim,
            icon_accent: accent,

            element_background: surface_2,
            element_hover: hover,
            element_active: active,
            element_selected: accent_bg,
            element_disabled: surface,

            ghost_element_background: TRANSPARENT,
            ghost_element_hover: oklch_to_hsla(1.0, 0.0, 0.0, 0.06),
            ghost_element_active: oklch_to_hsla(1.0, 0.0, 0.0, 0.10),
            ghost_element_selected: oklch_to_hsla(1.0, 0.0, 0.0, 0.08),
            ghost_element_disabled: TRANSPARENT,

            status: StatusColors {
                info,
                info_background: info.opacity(0.18),
                info_border: info.opacity(0.55),

                success,
                success_background: success.opacity(0.18),
                success_border: success.opacity(0.55),

                warning,
                warning_background: warning.opacity(0.18),
                warning_border: warning.opacity(0.55),

                error,
                error_background: error.opacity(0.18),
                error_border: error.opacity(0.55),

                hint: info,
                hint_background: info.opacity(0.18),
                hint_border: info.opacity(0.55),

                hidden,
                hidden_background: hidden.opacity(0.18),
                hidden_border: hidden.opacity(0.55),

                ignored,
                ignored_background: ignored.opacity(0.18),
                ignored_border: ignored.opacity(0.55),
            },

            accent,
        },
    }
}

/// The default light theme - warm-tinted surfaces with an amber accent,
/// tuned from the engram design draft.
pub fn light() -> Theme {
    let background = oklch(0.985, 0.004, 85.0);
    let surface = oklch(0.995, 0.004, 85.0);
    let surface_2 = oklch(0.97, 0.004, 85.0);

    let border = oklch(0.90, 0.006, 80.0);
    let border_strong = oklch(0.80, 0.008, 80.0);

    let fg = oklch(0.20, 0.006, 70.0);
    let fg_2 = oklch(0.36, 0.006, 70.0);
    let fg_muted = oklch(0.54, 0.006, 70.0);
    let fg_dim = oklch(0.72, 0.006, 75.0);

    let hover = oklch(0.94, 0.005, 80.0);
    let active = oklch(0.91, 0.006, 80.0);

    // Mono accent (near-fg) per draft's `data-accent="mono"` light variant.
    let accent = oklch(0.18, 0.004, 70.0);
    let accent_bg = oklch(0.92, 0.004, 80.0);

    let info = oklch(0.48, 0.12, 235.0);
    let success = oklch(0.48, 0.14, 150.0);
    let warning = oklch(0.55, 0.14, 70.0);
    let error = oklch(0.50, 0.17, 25.0);

    let hidden = oklch(0.64, 0.004, 75.0);
    let ignored = oklch(0.68, 0.004, 75.0);

    Theme {
        name: SharedString::new_static("Engram Light"),
        appearance: Appearance::Light,
        colors: ThemeColors {
            background,
            surface_background: surface,
            elevated_surface_background: surface_2,

            border,
            border_variant: oklch(0.94, 0.005, 80.0),
            border_focused: accent,
            border_selected: border_strong,
            border_disabled: oklch(0.94, 0.005, 80.0),
            border_transparent: TRANSPARENT,

            text: fg,
            text_muted: fg_2,
            text_placeholder: fg_muted,
            text_disabled: fg_dim,
            text_accent: accent,

            icon: fg,
            icon_muted: fg_muted,
            icon_disabled: fg_dim,
            icon_accent: accent,

            element_background: surface_2,
            element_hover: hover,
            element_active: active,
            element_selected: accent_bg,
            element_disabled: surface_2,

            ghost_element_background: TRANSPARENT,
            ghost_element_hover: oklch_to_hsla(0.0, 0.0, 0.0, 0.04),
            ghost_element_active: oklch_to_hsla(0.0, 0.0, 0.0, 0.08),
            ghost_element_selected: oklch_to_hsla(0.0, 0.0, 0.0, 0.06),
            ghost_element_disabled: TRANSPARENT,

            status: StatusColors {
                info,
                info_background: info.opacity(0.14),
                info_border: info.opacity(0.45),

                success,
                success_background: success.opacity(0.14),
                success_border: success.opacity(0.45),

                warning,
                warning_background: warning.opacity(0.14),
                warning_border: warning.opacity(0.45),

                error,
                error_background: error.opacity(0.14),
                error_border: error.opacity(0.45),

                hint: info,
                hint_background: info.opacity(0.14),
                hint_border: info.opacity(0.45),

                hidden,
                hidden_background: hidden.opacity(0.14),
                hidden_border: hidden.opacity(0.45),

                ignored,
                ignored_background: ignored.opacity(0.14),
                ignored_border: ignored.opacity(0.45),
            },

            accent,
        },
    }
}
