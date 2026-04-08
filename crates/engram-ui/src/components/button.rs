//! Button family: text [`Button`], icon-only [`IconButton`], and the shared
//! [`ButtonLike`] chrome they both compose.
//!
//! See `button/button_like.rs` for the architectural notes — this file is
//! just the module wiring.

#[allow(clippy::module_inception)]
mod button;
mod button_like;
mod icon_button;

pub use button::Button;
pub use button_like::{
    ButtonCommon, ButtonLike, ButtonSize, ButtonStyle, SelectableButton, TintColor,
};
pub use icon_button::IconButton;
