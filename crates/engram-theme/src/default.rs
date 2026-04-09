//! Hand-tuned default themes.
//!
//! The defaults are deliberately **neutral grayscale** — modeled after
//! shadcn/ui's "neutral" color scheme. There is no chromatic accent baked
//! into the base palette: `accent`, `text_accent`, `border_focused`, and
//! the element/border state ramps are all pure grays. The only colored
//! tokens in the default themes are the four severity foregrounds
//! (`status.{info,success,warning,error}`) and their alpha-tinted
//! `_background` / `_border` siblings — without those, `Banner` /
//! `Notification` / `Indicator` would lose their only signal.
//!
//! Themes that want a chromatic accent (gruvbox, solarized, …) override
//! the neutral defaults via JSON or by building a fresh `Theme`.

use gpui::SharedString;

use crate::{Appearance, Theme};
use crate::colors::{StatusColors, ThemeColors, hsl, hsla};

/// The default dark theme — shadcn neutral grayscale base, status colors
/// kept tinted so severity surfaces remain readable.
pub fn dark() -> Theme {
    Theme {
        name: SharedString::new_static("Engram Dark"),
        appearance: Appearance::Dark,
        colors: ThemeColors {
            // Three steps of neutral surface, matching shadcn neutral
            // background → card → popover/secondary.
            background: hsl(0.0, 0.00, 0.145),
            surface_background: hsl(0.0, 0.00, 0.205),
            elevated_surface_background: hsl(0.0, 0.00, 0.269),

            // shadcn neutral uses a translucent white border so it reads
            // against any of the three surface tones.
            border: hsla(0.0, 0.00, 1.00, 0.10),
            border_variant: hsla(0.0, 0.00, 1.00, 0.06),
            // The "ring" tone (mid-gray) gives focus a visible identity
            // distinct from accent so a focused-but-not-selected element
            // still reads correctly.
            border_focused: hsl(0.0, 0.00, 0.556),
            border_selected: hsl(0.0, 0.00, 0.556),
            border_disabled: hsla(0.0, 0.00, 1.00, 0.06),
            border_transparent: hsla(0.0, 0.00, 0.00, 0.00),

            text: hsl(0.0, 0.00, 0.985),
            text_muted: hsl(0.0, 0.00, 0.708),
            text_placeholder: hsl(0.0, 0.00, 0.50),
            text_disabled: hsl(0.0, 0.00, 0.40),
            // accent_text == text in the neutral scheme — accent surfaces
            // are high-contrast against their background, so the foreground
            // is the same near-white as ordinary body text.
            text_accent: hsl(0.0, 0.00, 0.985),

            icon: hsl(0.0, 0.00, 0.92),
            icon_muted: hsl(0.0, 0.00, 0.65),
            icon_disabled: hsl(0.0, 0.00, 0.40),
            icon_accent: hsl(0.0, 0.00, 0.985),

            element_background: hsl(0.0, 0.00, 0.205),
            element_hover: hsl(0.0, 0.00, 0.269),
            element_active: hsl(0.0, 0.00, 0.32),
            element_selected: hsl(0.0, 0.00, 0.269),
            element_disabled: hsl(0.0, 0.00, 0.18),

            ghost_element_background: hsla(0.0, 0.00, 0.00, 0.00),
            ghost_element_hover: hsla(0.0, 0.00, 1.00, 0.06),
            ghost_element_active: hsla(0.0, 0.00, 1.00, 0.10),
            ghost_element_selected: hsla(0.0, 0.00, 1.00, 0.08),
            ghost_element_disabled: hsla(0.0, 0.00, 0.00, 0.00),

            // Severity foregrounds keep their hue. The `_background` and
            // `_border` slots are alpha-tinted versions of the foreground
            // (the same formula engram has used since v0.1) so Banner /
            // Chip / Notification surfaces still differentiate by severity.
            status: StatusColors {
                info: hsl(210.0, 0.90, 0.60),
                info_background: hsla(210.0, 0.90, 0.60, 0.18),
                info_border: hsla(210.0, 0.90, 0.60, 0.55),

                success: hsl(142.0, 0.70, 0.50),
                success_background: hsla(142.0, 0.70, 0.50, 0.18),
                success_border: hsla(142.0, 0.70, 0.50, 0.55),

                warning: hsl(38.0, 0.90, 0.55),
                warning_background: hsla(38.0, 0.90, 0.55, 0.18),
                warning_border: hsla(38.0, 0.90, 0.55, 0.55),

                error: hsl(0.0, 0.70, 0.60),
                error_background: hsla(0.0, 0.70, 0.60, 0.18),
                error_border: hsla(0.0, 0.70, 0.60, 0.55),

                hint: hsl(210.0, 0.80, 0.65),
                hint_background: hsla(210.0, 0.80, 0.65, 0.18),
                hint_border: hsla(210.0, 0.80, 0.65, 0.55),

                hidden: hsl(0.0, 0.00, 0.45),
                hidden_background: hsla(0.0, 0.00, 0.45, 0.18),
                hidden_border: hsla(0.0, 0.00, 0.45, 0.55),

                ignored: hsl(0.0, 0.00, 0.38),
                ignored_background: hsla(0.0, 0.00, 0.38, 0.18),
                ignored_border: hsla(0.0, 0.00, 0.38, 0.55),
            },

            // Maps to shadcn neutral's "primary" — near-white in dark mode.
            // Used as the fill for accented surfaces (Indicator, CountBadge,
            // selected button styles); pairs with `text_accent` above.
            accent: hsl(0.0, 0.00, 0.922),
        },
    }
}

/// The default light theme — shadcn neutral grayscale base, status colors
/// kept tinted so severity surfaces remain readable.
pub fn light() -> Theme {
    Theme {
        name: SharedString::new_static("Engram Light"),
        appearance: Appearance::Light,
        colors: ThemeColors {
            background: hsl(0.0, 0.00, 1.00),
            surface_background: hsl(0.0, 0.00, 0.98),
            elevated_surface_background: hsl(0.0, 0.00, 1.00),

            border: hsl(0.0, 0.00, 0.922),
            border_variant: hsl(0.0, 0.00, 0.94),
            border_focused: hsl(0.0, 0.00, 0.708),
            border_selected: hsl(0.0, 0.00, 0.708),
            border_disabled: hsl(0.0, 0.00, 0.95),
            border_transparent: hsla(0.0, 0.00, 0.00, 0.00),

            text: hsl(0.0, 0.00, 0.145),
            text_muted: hsl(0.0, 0.00, 0.556),
            text_placeholder: hsl(0.0, 0.00, 0.65),
            text_disabled: hsl(0.0, 0.00, 0.75),
            text_accent: hsl(0.0, 0.00, 0.145),

            icon: hsl(0.0, 0.00, 0.20),
            icon_muted: hsl(0.0, 0.00, 0.50),
            icon_disabled: hsl(0.0, 0.00, 0.75),
            icon_accent: hsl(0.0, 0.00, 0.145),

            element_background: hsl(0.0, 0.00, 0.97),
            element_hover: hsl(0.0, 0.00, 0.94),
            element_active: hsl(0.0, 0.00, 0.90),
            element_selected: hsl(0.0, 0.00, 0.92),
            element_disabled: hsl(0.0, 0.00, 0.96),

            ghost_element_background: hsla(0.0, 0.00, 0.00, 0.00),
            ghost_element_hover: hsla(0.0, 0.00, 0.00, 0.04),
            ghost_element_active: hsla(0.0, 0.00, 0.00, 0.08),
            ghost_element_selected: hsla(0.0, 0.00, 0.00, 0.06),
            ghost_element_disabled: hsla(0.0, 0.00, 0.00, 0.00),

            status: StatusColors {
                info: hsl(210.0, 0.85, 0.48),
                info_background: hsla(210.0, 0.85, 0.48, 0.14),
                info_border: hsla(210.0, 0.85, 0.48, 0.45),

                success: hsl(142.0, 0.65, 0.38),
                success_background: hsla(142.0, 0.65, 0.38, 0.14),
                success_border: hsla(142.0, 0.65, 0.38, 0.45),

                warning: hsl(38.0, 0.90, 0.48),
                warning_background: hsla(38.0, 0.90, 0.48, 0.14),
                warning_border: hsla(38.0, 0.90, 0.48, 0.45),

                error: hsl(0.0, 0.70, 0.50),
                error_background: hsla(0.0, 0.70, 0.50, 0.14),
                error_border: hsla(0.0, 0.70, 0.50, 0.45),

                hint: hsl(210.0, 0.85, 0.45),
                hint_background: hsla(210.0, 0.85, 0.45, 0.14),
                hint_border: hsla(210.0, 0.85, 0.45, 0.45),

                hidden: hsl(0.0, 0.00, 0.62),
                hidden_background: hsla(0.0, 0.00, 0.62, 0.14),
                hidden_border: hsla(0.0, 0.00, 0.62, 0.45),

                ignored: hsl(0.0, 0.00, 0.68),
                ignored_background: hsla(0.0, 0.00, 0.68, 0.14),
                ignored_border: hsla(0.0, 0.00, 0.68, 0.45),
            },

            accent: hsl(0.0, 0.00, 0.205),
        },
    }
}
