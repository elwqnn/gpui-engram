//! Unit tests for JSON theme loading, refinement, and registry bookkeeping.

use gpui::{Hsla, Rgba};
use gpui_engram_theme::{
    Appearance, ThemeColorsRefinement, ThemeContent, ThemeRegistry, default_dark, default_light,
};

fn hex(rgba: &str) -> Hsla {
    // Round-trip through the gpui Rgba Deserialize impl (which accepts
    // `#RGB` / `#RGBA` / `#RRGGBB` / `#RRGGBBAA`) and into an Hsla.
    let json = format!("\"{rgba}\"");
    let rgba: Rgba = serde_json::from_str(&json).unwrap();
    rgba.into()
}

#[test]
fn parses_a_full_theme_document() {
    let json = r##"{
        "name": "Test Dark",
        "appearance": "dark",
        "colors": {
            "background": "#0a0a0a",
            "text": "#eeeeee",
            "accent": "#4a9eff"
        }
    }"##;

    let theme = gpui_engram_theme::Theme::from_json_str(json).unwrap();
    assert_eq!(theme.name.as_ref(), "Test Dark");
    assert_eq!(theme.appearance, Appearance::Dark);
    assert_eq!(theme.colors.background, hex("#0a0a0a"));
    assert_eq!(theme.colors.text, hex("#eeeeee"));
    assert_eq!(theme.colors.accent, hex("#4a9eff"));
}

#[test]
fn fields_not_in_the_json_fall_through_to_the_base_theme() {
    let base = default_dark();
    let json = r##"{
        "name": "Partial",
        "appearance": "dark",
        "colors": {
            "accent": "#ff00ff"
        }
    }"##;

    let theme = gpui_engram_theme::Theme::from_json_str(json).unwrap();
    assert_eq!(theme.colors.accent, hex("#ff00ff"));
    // Every other token should match the built-in dark theme byte-for-byte.
    assert_eq!(theme.colors.background, base.colors.background);
    assert_eq!(theme.colors.text, base.colors.text);
    assert_eq!(theme.colors.status.info, base.colors.status.info);
}

#[test]
fn partial_status_overrides_leave_siblings_alone() {
    let base = default_dark();
    let json = r##"{
        "name": "Partial Status",
        "appearance": "dark",
        "colors": {
            "status": {
                "error": "#ff1111"
            }
        }
    }"##;
    let theme = gpui_engram_theme::Theme::from_json_str(json).unwrap();
    assert_eq!(theme.colors.status.error, hex("#ff1111"));
    assert_eq!(theme.colors.status.success, base.colors.status.success);
    assert_eq!(theme.colors.status.warning, base.colors.status.warning);
    // Non-status fields still fall through.
    assert_eq!(theme.colors.accent, base.colors.accent);
}

#[test]
fn unknown_top_level_fields_are_rejected() {
    let json = r##"{
        "name": "Bad",
        "appearance": "dark",
        "extra": "nope"
    }"##;
    let err = gpui_engram_theme::Theme::from_json_str(json).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("extra"), "unexpected error: {msg}");
}

#[test]
fn unknown_color_fields_are_rejected() {
    let json = r##"{
        "name": "Bad",
        "appearance": "dark",
        "colors": {
            "totally_fake": "#000000"
        }
    }"##;
    let err = gpui_engram_theme::Theme::from_json_str(json).unwrap_err();
    assert!(err.to_string().contains("totally_fake"));
}

#[test]
fn registry_insert_and_lookup() {
    let mut registry = ThemeRegistry::new();
    assert!(registry.is_empty());

    registry.insert(default_dark());
    registry.insert(default_light());
    assert_eq!(registry.len(), 2);

    let names = registry.names();
    assert!(names.iter().any(|n| n.as_ref() == "Engram Dark"));
    assert!(names.iter().any(|n| n.as_ref() == "Engram Light"));

    let dark = registry.get("Engram Dark").expect("dark registered");
    assert_eq!(dark.appearance, Appearance::Dark);
    assert!(registry.get("missing").is_none());
}

#[test]
fn round_trip_dark_theme_through_refinement() {
    // ThemeColorsRefinement::from_full followed by refine() should produce
    // the exact same ThemeColors as the input.
    let dark = default_dark();
    let refinement = ThemeColorsRefinement::from_full(&dark.colors);
    let mut base = default_light(); // start from a *different* theme.
    refinement.refine(&mut base.colors);
    assert_eq!(base.colors, dark.colors);
}

#[test]
fn round_trip_theme_content_json_is_stable_after_one_pass() {
    // The on-disk JSON format is 8-bit sRGB hex, so an Hsla going through
    // from_full -> serialize -> parse -> refine loses the lower bits of
    // the hsl coordinates. One round trip is enough to reach a stable
    // fixed point - a *second* serialize -> parse of the reparsed theme
    // must yield byte-identical JSON.
    let dark = default_dark();
    let json1 = serde_json::to_string_pretty(&ThemeContent::from_theme(&dark)).unwrap();
    let reparsed = gpui_engram_theme::Theme::from_json_str(&json1).unwrap();
    let json2 = serde_json::to_string_pretty(&ThemeContent::from_theme(&reparsed)).unwrap();
    assert_eq!(json1, json2);
    assert_eq!(reparsed.name, dark.name);
    assert_eq!(reparsed.appearance, dark.appearance);
}

/// Regenerates the bundled JSON fixtures under
/// `crates/engram-ui/assets/themes/`. Ignored by default - run explicitly
/// with `cargo test -p gpui-engram-theme --test loader -- --ignored regenerate_builtin_fixtures`
/// whenever the hand-tuned defaults in `engram-theme/src/default.rs`
/// change.
#[test]
fn accepts_hex_rgb_hsl_oklch_in_same_document() {
    let json = r##"{
        "name": "Mixed Formats",
        "appearance": "dark",
        "colors": {
            "background": "#1d2021",
            "text": "rgb(235 219 178)",
            "border": "hsl(40 20% 40%)",
            "accent": "oklch(0.72 0.09 235)"
        }
    }"##;

    let theme = gpui_engram_theme::Theme::from_json_str(json).unwrap();

    // Each format resolved; values are non-zero / non-default.
    assert_ne!(theme.colors.background, default_dark().colors.background);
    assert_ne!(theme.colors.text, default_dark().colors.text);
    assert_ne!(theme.colors.border, default_dark().colors.border);
    assert_ne!(theme.colors.accent, default_dark().colors.accent);

    // Hex round-trip sanity.
    assert_eq!(theme.colors.background, hex("#1d2021ff"));
}

#[test]
#[ignore = "regeneration helper - run manually"]
fn regenerate_builtin_fixtures() {
    let assets = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("engram-ui")
        .join("assets")
        .join("themes");
    std::fs::create_dir_all(&assets).unwrap();

    for (file, theme) in [
        ("engram_dark.json", default_dark()),
        ("engram_light.json", default_light()),
    ] {
        let content = ThemeContent::from_theme(&theme);
        let mut json = serde_json::to_string_pretty(&content).unwrap();
        json.push('\n');
        std::fs::write(assets.join(file), json).unwrap();
    }
}
