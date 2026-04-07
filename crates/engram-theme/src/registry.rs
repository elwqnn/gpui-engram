//! In-process catalog of loaded themes.
//!
//! [`ThemeRegistry`] lives as a GPUI [`Global`] and holds every theme the app
//! knows about — both the built-ins registered during [`crate::init`] and
//! anything the hot-reload watcher has picked up at runtime. Components and
//! example apps switch themes by name through this registry instead of
//! plumbing [`Theme`] values around manually.

use std::collections::BTreeMap;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use gpui::{App, Global, SharedString};

use crate::{Theme, set_theme};

/// A name-keyed collection of [`Theme`]s. Insertion-stable but iteration is
/// alphabetic so UIs can list themes without needing their own sort.
#[derive(Default)]
pub struct ThemeRegistry {
    themes: BTreeMap<SharedString, Arc<Theme>>,
}

impl Global for ThemeRegistry {}

impl ThemeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add (or replace) a theme. Returns the previous entry under the same
    /// name, if any — callers can ignore it for first-time inserts.
    pub fn insert(&mut self, theme: Theme) -> Option<Arc<Theme>> {
        let name = theme.name.clone();
        self.themes.insert(name, Arc::new(theme))
    }

    /// Look up a theme by name.
    pub fn get(&self, name: &str) -> Option<Arc<Theme>> {
        self.themes.get(name).cloned()
    }

    /// All loaded theme names, alphabetically sorted.
    pub fn names(&self) -> Vec<SharedString> {
        self.themes.keys().cloned().collect()
    }

    /// Iterator over `(name, theme)` pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&SharedString, &Arc<Theme>)> {
        self.themes.iter()
    }

    /// Number of registered themes.
    pub fn len(&self) -> usize {
        self.themes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.themes.is_empty()
    }

    /// Borrow the registry from a GPUI [`App`]. Panics if
    /// [`init`](crate::init) hasn't run.
    pub fn global(cx: &App) -> &Self {
        cx.global::<Self>()
    }

    /// Mutably borrow the registry from a GPUI [`App`]. Panics if
    /// [`init`](crate::init) hasn't run.
    pub fn global_mut(cx: &mut App) -> &mut Self {
        cx.global_mut::<Self>()
    }
}

/// Look up a theme in the global registry by name and install it as the
/// active theme. Returns an error if no theme is registered under that name.
pub fn activate_theme(name: &str, cx: &mut App) -> Result<()> {
    let theme = ThemeRegistry::global(cx)
        .get(name)
        .ok_or_else(|| anyhow!("no theme named `{name}` is registered"))?;
    set_theme((*theme).clone(), cx);
    Ok(())
}
