//! Bridge between the OS light/dark appearance and engram's active theme.
//!
//! When enabled, this module subscribes to GPUI's window appearance observer
//! and calls [`activate_theme`] with either the dark or the light theme name
//! whenever the system flips. This keeps apps that follow the OS scheme in
//! sync without each call site wiring its own observer.

use gpui::{App, SharedString, Subscription, Window, WindowAppearance};

use crate::registry::{ThemeRegistry, activate_theme};

/// Configuration for [`sync_with_system_appearance`] — which registered
/// theme name to use for each OS appearance bucket.
#[derive(Debug, Clone)]
pub struct SystemAppearanceConfig {
    pub dark_theme: SharedString,
    pub light_theme: SharedString,
}

impl Default for SystemAppearanceConfig {
    fn default() -> Self {
        Self {
            dark_theme: SharedString::new_static("Engram Dark"),
            light_theme: SharedString::new_static("Engram Light"),
        }
    }
}

impl SystemAppearanceConfig {
    fn theme_for(&self, appearance: WindowAppearance) -> &SharedString {
        match appearance {
            WindowAppearance::Dark | WindowAppearance::VibrantDark => &self.dark_theme,
            WindowAppearance::Light | WindowAppearance::VibrantLight => &self.light_theme,
        }
    }
}

/// Start mirroring the given [`Window`]'s OS appearance onto the active
/// theme. Immediately applies the current appearance, then installs an
/// observer that re-applies it on every change.
///
/// Returns a [`Subscription`] — keep it alive for as long as you want the
/// sync to run (dropping it unsubscribes).
///
/// If either the `dark_theme` or `light_theme` name is missing from the
/// [`ThemeRegistry`], the mismatched arm is silently ignored (the previous
/// theme stays active).
pub fn sync_with_system_appearance(
    config: SystemAppearanceConfig,
    window: &mut Window,
    cx: &mut App,
) -> Subscription {
    // Apply the current appearance up front, so the window matches the OS
    // from the very first frame.
    apply_appearance(&config, window.appearance(), cx);

    window.observe_window_appearance(move |window, cx| {
        apply_appearance(&config, window.appearance(), cx);
    })
}

fn apply_appearance(config: &SystemAppearanceConfig, appearance: WindowAppearance, cx: &mut App) {
    let target = config.theme_for(appearance).clone();
    if ThemeRegistry::global(cx).get(&target).is_some() {
        // Any error here means the registry dropped the theme between the
        // check and the call — shouldn't happen, but there's no useful
        // recovery, so swallow it.
        let _ = activate_theme(&target, cx);
    }
}
