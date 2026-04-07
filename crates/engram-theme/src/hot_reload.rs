//! Watch a directory of JSON theme files and reload them in-place when
//! they change on disk.
//!
//! The primary entry point is [`watch_themes_dir`]. It takes an absolute
//! path and, from that moment on, any `.json` file dropped into (or saved
//! over) that directory is parsed via [`Theme::from_json_bytes`] and merged
//! into the [`ThemeRegistry`]. If the file holds the currently-active
//! theme, the active theme is re-applied so the next frame reflects the
//! new colors.
//!
//! The actual filesystem notifications are delivered by `notify` on a
//! background thread. A GPUI foreground task polls the events and invokes
//! the update logic on the main thread, so reloads are always observed
//! with a live `App`.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

use anyhow::Result;
use gpui::{App, Task};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::registry::{ThemeRegistry, activate_theme};
use crate::{ActiveTheme, Theme};

/// Handle returned by [`watch_themes_dir`]. Keep it alive for as long as
/// hot reload should run — dropping it tears down the watcher and the
/// polling task.
pub struct ThemeWatcher {
    _watcher: RecommendedWatcher,
    _poll_task: Task<()>,
}

/// Start watching `dir` for JSON theme changes. Any pre-existing `.json`
/// files are loaded immediately; new files are picked up as they appear.
///
/// Errors if `dir` does not exist, is not a directory, or if the watcher
/// cannot be installed (e.g. on platforms without an inotify backend).
pub fn watch_themes_dir(dir: impl AsRef<Path>, cx: &mut App) -> Result<ThemeWatcher> {
    let dir = dir.as_ref().to_path_buf();
    if !dir.is_dir() {
        anyhow::bail!("theme directory does not exist: {}", dir.display());
    }

    // Load everything already on disk once, so the registry reflects the
    // contents of the directory even before the first filesystem event.
    load_all_themes(&dir, cx);

    // notify fires events from a background thread — forward them onto an
    // mpsc channel that the foreground poll task drains.
    let (tx, rx) = channel::<notify::Result<Event>>();
    let mut watcher = notify::recommended_watcher(move |res| {
        let _ = tx.send(res);
    })?;
    watcher.watch(&dir, RecursiveMode::NonRecursive)?;

    let poll_task = spawn_poll_task(dir, rx, cx);

    Ok(ThemeWatcher {
        _watcher: watcher,
        _poll_task: poll_task,
    })
}

fn spawn_poll_task(
    dir: PathBuf,
    rx: Receiver<notify::Result<Event>>,
    cx: &mut App,
) -> Task<()> {
    cx.spawn(async move |cx| {
        loop {
            // Drain every event that's queued up since the last tick.
            let mut touched = false;
            while let Ok(event) = rx.try_recv() {
                if let Ok(ev) = event
                    && is_relevant(&ev)
                {
                    touched = true;
                }
            }
            if touched {
                let dir = dir.clone();
                cx.update(|cx| load_all_themes(&dir, cx));
            }

            cx.background_executor()
                .timer(Duration::from_millis(150))
                .await;
        }
    })
}

fn is_relevant(event: &Event) -> bool {
    matches!(
        event.kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    ) && event
        .paths
        .iter()
        .any(|p| p.extension().map(|e| e == "json").unwrap_or(false))
}

fn load_all_themes(dir: &Path, cx: &mut App) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map(|e| e != "json").unwrap_or(true) {
            continue;
        }
        match load_single(&path) {
            Ok(theme) => {
                let name = theme.name.clone();
                ThemeRegistry::global_mut(cx).insert(theme);
                // If the reloaded theme is the one currently active, re-activate
                // it so the next frame paints with the new colors.
                if cx.theme().name == name {
                    let _ = activate_theme(&name, cx);
                }
            }
            Err(err) => {
                eprintln!(
                    "engram-theme: failed to reload `{}`: {err}",
                    path.display()
                );
            }
        }
    }
}

fn load_single(path: &Path) -> Result<Theme> {
    let bytes = std::fs::read(path)?;
    Theme::from_json_bytes(&bytes)
}
