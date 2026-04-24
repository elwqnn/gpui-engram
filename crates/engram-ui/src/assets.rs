//! Embedded asset bundle for engram-ui (icons today, fonts/images later).
//!
//! [`Assets`] implements gpui's [`AssetSource`] so callers can wire it into
//! their `Application` with `application().with_assets(gpui_engram_ui::Assets)`.
//! Once registered, every [`Icon`](crate::components::Icon) resolves its SVG
//! through this source.

use std::borrow::Cow;

use anyhow::Context as _;
use gpui::{AssetSource, Result, SharedString};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "assets"]
#[include = "icons/**/*.svg"]
#[include = "brand/*.svg"]
#[include = "themes/*.json"]
#[include = "fonts/*.ttf"]
#[exclude = "*.DS_Store"]
pub struct Assets;

/// Bundled font files, shipped with engram-ui and registered by
/// [`load_bundled_fonts`]. Three families: **Funnel Sans** (sans UI default),
/// **JetBrains Mono** (mono for KeybindingHint, code in tooltips/callouts),
/// and **Lora** (serif, available for consumers that want it).
///
/// Only the weights exercised by the component library (400/500/600, and
/// their italics) are listed here; additional weights sitting in
/// `assets/fonts/` are still embedded by `rust-embed` and can be loaded
/// ad-hoc via [`Assets::load`] if an app needs them.
pub const BUNDLED_FONTS: &[&str] = &[
    "fonts/FunnelSans-Regular.ttf",
    "fonts/FunnelSans-Italic.ttf",
    "fonts/FunnelSans-Medium.ttf",
    "fonts/FunnelSans-MediumItalic.ttf",
    "fonts/FunnelSans-SemiBold.ttf",
    "fonts/FunnelSans-SemiBoldItalic.ttf",
    "fonts/JetBrainsMono-Regular.ttf",
    "fonts/JetBrainsMono-Italic.ttf",
    "fonts/JetBrainsMono-Medium.ttf",
    "fonts/JetBrainsMono-MediumItalic.ttf",
    "fonts/JetBrainsMono-SemiBold.ttf",
    "fonts/JetBrainsMono-SemiBoldItalic.ttf",
    "fonts/Lora-Regular.ttf",
    "fonts/Lora-Italic.ttf",
    "fonts/Lora-Medium.ttf",
    "fonts/Lora-MediumItalic.ttf",
    "fonts/Lora-SemiBold.ttf",
    "fonts/Lora-SemiBoldItalic.ttf",
];

/// Load every font in [`BUNDLED_FONTS`] into the gpui text system. Called
/// from [`crate::init`]; exposed so an app that bypasses `init` can opt in
/// explicitly. Missing embedded files are ignored (logged via `eprintln!`)
/// so a stripped-down build that doesn't ship a given weight still starts.
pub fn load_bundled_fonts(cx: &gpui::App) {
    let mut bytes: Vec<std::borrow::Cow<'static, [u8]>> = Vec::new();
    for path in BUNDLED_FONTS {
        match Assets.load(path) {
            Ok(Some(data)) => bytes.push(data),
            Ok(None) => eprintln!("engram-ui: bundled font missing: {path}"),
            Err(err) => eprintln!("engram-ui: failed to load bundled font {path}: {err}"),
        }
    }
    if let Err(err) = cx.text_system().add_fonts(bytes) {
        eprintln!("engram-ui: add_fonts failed: {err}");
    }
}

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        Self::get(path)
            .map(|f| Some(f.data))
            .with_context(|| format!("loading asset at path {path:?}"))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(Self::iter()
            .filter_map(|p| p.starts_with(path).then(|| p.into()))
            .collect())
    }
}
