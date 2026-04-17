//! engram-ui: GPUI component primitives built on top of [`engram_theme`].
//!
//! Downstream users should typically `use engram::prelude::*;` — the umbrella
//! [`engram`](../engram/index.html) crate re-exports both this crate and
//! [`engram_theme`].
//!
//! Apps should call [`init`] during startup (alongside `engram_theme::init`)
//! so the built-in components can register their default keybindings.

#![forbid(unsafe_code)]

pub mod assets;
pub mod components;
pub mod styles;
pub mod traits;

pub use anyhow::{Error, Result};

pub use assets::Assets;
pub use components::*;
pub use styles::ElevationIndex;
pub use traits::*;

/// Initialize engram-ui's process-global state.
///
/// Registers the default keybindings for components that need them — today
/// that's [`components::TextField`] (arrow navigation, selection, clipboard,
/// submit) and [`components::Menu`] (arrow navigation, Enter / Escape). Call
/// it once per `App`, after [`engram_theme::init`] and before rendering any
/// components that depend on those bindings.
pub fn init(cx: &mut gpui::App) {
    components::text_field::bind_text_field_keys(cx);
    components::menu::bind_menu_keys(cx);
}

/// Re-exports of the things you almost always want when building an engram UI.
pub mod prelude {
    pub use engram_theme::{
        ActiveTheme, Appearance, Color, Radius, Spacing, TextSize, Theme, ThemeColors,
    };

    pub use crate::assets::Assets;
    pub use crate::components::{
        Accordion, AccordionItem, Avatar, AvatarSize, Banner, BorderPosition, Breadcrumb,
        BreadcrumbItem, Button, ButtonBuilder, ButtonCommon, ButtonLike, ButtonLink, ButtonSize,
        ButtonStyle, Callout, Checkbox, CheckboxSize, Chip, ChipSize, ChipStyle, CircularProgress,
        CopyButton, CountBadge, DecoratedIcon, DescriptionEntry, DescriptionList, Disclosure,
        Divider, DividerOrientation, DropdownMenu, EndSlotVisibility, Facepile, GradientFade,
        Headline, HeadlineSize, HighlightedLabel, HoverCard, Icon, IconButton, IconDecoration,
        IconName, IconSize, IconSource, Image, Indicator, KeyBinding, KeybindingHint, Label,
        LabelCommon, LabelLike, LabelSize, LineHeightStyle, List, ListItem, ListItemSpacing, Menu,
        MenuItem, Modal, Navigable, NavigableEntry, Notification, Pagination, Popover, ProgressBar,
        Radio, ScrollStrategy, Scrollbar, ScrollbarAxis, SelectableButton, Severity, Sheet,
        SheetSide, Skeleton, SkeletonShape, Slider, Spinner, SplitButton, SplitButtonKind,
        SplitButtonStyle, Stepper, Switch, Tab, TabBar, TextField, TextFieldSubmitEvent, TintColor,
        ToggleButtonGroup, ToggleButtonGroupStyle, ToggleButtonSimple, ToggleButtonWithIcon,
        Tooltip, TreeViewItem, VariableList, VariableListAlignment, VariableListScrollHandle,
        VirtualList, VirtualListScrollHandle, anchored_popover, h_flex, h_group, h_group_lg,
        modal_overlay, sheet_overlay, skeleton_text, text_field, v_flex, v_group, v_group_lg,
    };
    pub use crate::styles::ElevationIndex;
    pub use crate::traits::{Clickable, Disableable, StyledExt, ToggleState, Toggleable};
}
