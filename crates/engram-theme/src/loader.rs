//! JSON theme loading.
//!
//! A [`ThemeContent`] is the on-disk representation of a theme: a name, an
//! appearance hint, and a partial [`ThemeColorsRefinement`]. The appearance
//! field decides which built-in base theme (`default_dark` or `default_light`)
//! the refinement is layered on top of, so user themes can be as sparse as
//! they like - overriding a single accent color is a valid theme.
//!
//! The canonical entry point is [`Theme::from_json_bytes`], used both by the
//! built-in theme assets and by [`hot_reload`](crate::hot_reload) when it
//! spots a file change.

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

use crate::default::{dark as default_dark, light as default_light};
use crate::refinement::ThemeColorsRefinement;
use crate::{Appearance, Theme};

/// Current theme JSON schema version. Incremented on breaking changes to
/// the on-disk format. Files without a `schema_version` field are assumed
/// to be v1 for backward compatibility with themes written before the
/// field was introduced.
pub const THEME_SCHEMA_VERSION: u32 = 1;

fn default_schema_version() -> u32 {
    THEME_SCHEMA_VERSION
}

/// On-disk representation of a theme. Deserialized from JSON.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ThemeContent {
    /// Schema version this document was written against. Defaults to the
    /// current [`THEME_SCHEMA_VERSION`] when absent.
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    pub name: String,
    pub appearance: AppearanceContent,
    #[serde(default)]
    pub colors: ThemeColorsRefinement,
}

/// JSON spelling of [`Appearance`]. Lowercase (`"light"` / `"dark"`).
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AppearanceContent {
    Light,
    Dark,
}

impl From<Appearance> for AppearanceContent {
    fn from(value: Appearance) -> Self {
        match value {
            Appearance::Light => AppearanceContent::Light,
            Appearance::Dark => AppearanceContent::Dark,
        }
    }
}

impl From<AppearanceContent> for Appearance {
    fn from(value: AppearanceContent) -> Self {
        match value {
            AppearanceContent::Light => Appearance::Light,
            AppearanceContent::Dark => Appearance::Dark,
        }
    }
}

impl ThemeContent {
    /// Materialize this content into a full [`Theme`] by layering its
    /// refinement on top of the matching built-in base theme.
    pub fn into_theme(self) -> Theme {
        let mut base = match self.appearance {
            AppearanceContent::Dark => default_dark(),
            AppearanceContent::Light => default_light(),
        };
        self.colors.refine(&mut base.colors);
        Theme {
            name: self.name.into(),
            appearance: self.appearance.into(),
            colors: base.colors,
        }
    }

    /// Build a fully-populated content document from an in-memory theme.
    /// Used to dump the built-in themes to canonical JSON fixtures that
    /// users can copy and edit.
    pub fn from_theme(theme: &Theme) -> Self {
        Self {
            schema_version: THEME_SCHEMA_VERSION,
            name: theme.name.to_string(),
            appearance: theme.appearance.into(),
            colors: ThemeColorsRefinement::from_full(&theme.colors),
        }
    }
}

impl Theme {
    /// Parse a theme from JSON bytes. The JSON document must have a `name`,
    /// an `appearance` (`"light"` or `"dark"`), and an optional `colors`
    /// object containing partial overrides.
    ///
    /// An optional `schema_version` field is honored if present. Documents
    /// written against a version *newer* than [`THEME_SCHEMA_VERSION`] are
    /// rejected - this crate does not guess at future fields.
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self> {
        let content: ThemeContent = serde_json::from_slice(bytes)?;
        if content.schema_version == 0 {
            return Err(anyhow!(
                "theme `{}` declares schema_version 0; versions start at 1",
                content.name
            ));
        }
        if content.schema_version > THEME_SCHEMA_VERSION {
            return Err(anyhow!(
                "theme `{}` declares schema_version {} but this crate only understands up to {}",
                content.name,
                content.schema_version,
                THEME_SCHEMA_VERSION
            ));
        }
        Ok(content.into_theme())
    }

    /// Parse a theme from a JSON string. Convenience over
    /// [`Theme::from_json_bytes`].
    pub fn from_json_str(json: &str) -> Result<Self> {
        Self::from_json_bytes(json.as_bytes())
    }
}
