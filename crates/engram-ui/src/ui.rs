//! engram-ui: GPUI component primitives built on top of [`engram_theme`].
//!
//! Downstream users should typically `use engram::prelude::*;` — the umbrella
//! [`engram`](../engram/index.html) crate re-exports both this crate and
//! [`engram_theme`].
//!
//! Apps should call [`init`] during startup (alongside `engram_theme::init`)
//! so the built-in components can register their default keybindings.

pub mod assets;
pub mod components;
pub mod styles;
pub mod traits;

pub use assets::Assets;
pub use components::*;
pub use styles::ElevationIndex;
pub use traits::*;

/// Initialize engram-ui's process-global state.
///
/// Today this just registers the default [`components::TextField`]
/// keybindings (arrow navigation, selection, clipboard, submit). Call it
/// once per `App`, after [`engram_theme::init`] and before rendering any
/// components that depend on those bindings.
pub fn init(cx: &mut gpui::App) {
    components::text_field::bind_text_field_keys(cx);
}

/// Re-exports of the things you almost always want when building an engram UI.
pub mod prelude {
    pub use engram_theme::{
        ActiveTheme, Appearance, Color, Radius, Spacing, TextSize, Theme, ThemeColors,
    };

    pub use crate::assets::Assets;
    pub use crate::components::{
        Avatar, AvatarSize, Banner, Button, ButtonCommon, ButtonLike, ButtonSize, ButtonStyle,
        Checkbox, CheckboxSize, Chip, ChipStyle, CountBadge, Disclosure, Divider,
        DividerOrientation, Facepile, Icon, IconButton, IconName, IconSize, Image, Indicator,
        KeyBinding, Label, List, ListItem, Menu, MenuItem, Modal, Notification, Popover,
        Scrollbar, ScrollbarAxis, SelectableButton, Severity, Switch, Tab, TabBar, TextField,
        TextFieldSubmitEvent, TintColor, Tooltip, anchored_popover, h_flex, modal_overlay,
        text_field, v_flex,
    };
    pub use crate::styles::ElevationIndex;
    pub use crate::traits::{Clickable, Disableable, StyledExt, ToggleState, Toggleable};
}
