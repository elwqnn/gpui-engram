//! Embedded asset bundle for engram-ui (icons today, fonts/images later).
//!
//! [`Assets`] implements gpui's [`AssetSource`] so callers can wire it into
//! their `Application` with `application().with_assets(engram_ui::Assets)`.
//! Once registered, every [`Icon`](crate::components::Icon) resolves its SVG
//! through this source.

use std::borrow::Cow;

use anyhow::Context as _;
use gpui::{AssetSource, Result, SharedString};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "assets"]
#[include = "icons/**/*.svg"]
#[include = "themes/*.json"]
#[exclude = "*.DS_Store"]
pub struct Assets;

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
