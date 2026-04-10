//! Thin wrapper around [`gpui::img`] for showing bitmap / vector images.
//!
//! GPUI's `img()` element already handles loading from URLs, file paths,
//! and pre-loaded [`Arc<Image>`](gpui::Image) sources. This wrapper exists
//! to:
//!
//! - give callers a consistent `Image::new` constructor alongside the other
//!   engram components,
//! - bake engram's sizing and rounding tokens in through the type system
//!   ([`Radius`]), and
//! - give us a place to hang future defaults (loading spinner, error
//!   fallback) without changing the downstream call sites.
//!
//! For a circular profile picture, prefer
//! [`Avatar::image`](crate::components::Avatar::image) — it sizes and clips
//! the image to match the rest of the avatar family.

use std::path::Path;
use std::sync::Arc;

use engram_theme::Radius;
use gpui::{
    App, ImageSource, IntoElement, ObjectFit, Pixels, RenderImage, RenderOnce, Styled, StyledImage,
    Window, img,
};

/// A styled image element.
///
/// Simple builder around [`gpui::img`]. All fields default to "unset" —
/// if neither `width` nor `height` is given, the image lays out at its
/// intrinsic size.
#[derive(IntoElement)]
pub struct Image {
    source: ImageSource,
    width: Option<Pixels>,
    height: Option<Pixels>,
    radius: Option<Radius>,
    object_fit: ObjectFit,
    grayscale: bool,
}

impl Image {
    pub fn new(source: impl Into<ImageSource>) -> Self {
        Self {
            source: source.into(),
            width: None,
            height: None,
            radius: None,
            // GPUI's shader applies corner_radii to the sprite's *painted*
            // bounds. With Cover those bounds extend beyond the layout area,
            // making rounding invisible. Zed uses Contain everywhere for the
            // same reason — the image fits within its layout bounds and
            // rounding works correctly.
            object_fit: ObjectFit::Contain,
            grayscale: false,
        }
    }

    /// Set both width and height to the same value (useful for square
    /// thumbnails).
    pub fn size(mut self, size: Pixels) -> Self {
        self.width = Some(size);
        self.height = Some(size);
        self
    }

    pub fn width(mut self, width: Pixels) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: Pixels) -> Self {
        self.height = Some(height);
        self
    }

    /// Round the image's corners using an engram [`Radius`] token.
    pub fn rounded(mut self, radius: Radius) -> Self {
        self.radius = Some(radius);
        self
    }

    /// Fully round the image (convenience for circular avatars).
    pub fn rounded_full(self) -> Self {
        self.rounded(Radius::Full)
    }

    /// Override the default [`ObjectFit::Cover`] behavior.
    pub fn object_fit(mut self, object_fit: ObjectFit) -> Self {
        self.object_fit = object_fit;
        self
    }

    pub fn grayscale(mut self, grayscale: bool) -> Self {
        self.grayscale = grayscale;
        self
    }
}

impl RenderOnce for Image {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // Radius::Full (circle) needs the image to fill the square exactly
        // so the circular SDF clips a full disk. Fill stretches slightly but
        // inside a small circle the distortion is imperceptible — this is how
        // every avatar system works. Other radii keep the caller's ObjectFit.
        let object_fit = match self.radius {
            Some(Radius::Full) => ObjectFit::Fill,
            _ => self.object_fit,
        };

        let mut image = img(self.source).object_fit(object_fit);
        if let Some(w) = self.width {
            image = image.w(w);
        }
        if let Some(h) = self.height {
            image = image.h(h);
        }
        if let Some(r) = self.radius {
            image = image.rounded(r.pixels());
        }
        if self.grayscale {
            image = image.grayscale(true);
        }

        image
    }
}

/// Load an image from disk and center-crop it to the largest square.
///
/// Returns an [`ImageSource`] backed by pre-cropped pixel data. The crop
/// happens once at call time — after that the GPU texture is cached like
/// any other image. Use this when you need `rounded_full()` on a
/// non-square source image so the circle fills completely without
/// stretching.
pub fn center_crop_square(path: impl AsRef<Path>) -> anyhow::Result<ImageSource> {
    let data = std::fs::read(path.as_ref())?;
    let decoded = image::load_from_memory(&data)?;
    let rgba = decoded.to_rgba8();
    let (w, h) = rgba.dimensions();
    let side = w.min(h);
    let x = (w - side) / 2;
    let y = (h - side) / 2;
    let mut cropped = image::imageops::crop_imm(&rgba, x, y, side, side).to_image();
    // GPUI expects BGRA pixel order; the `image` crate produces RGBA.
    for pixel in cropped.as_flat_samples_mut().samples.chunks_exact_mut(4) {
        pixel.swap(0, 2);
    }
    let frame = image::Frame::new(cropped);
    let render = Arc::new(RenderImage::new(smallvec::smallvec![frame]));
    Ok(ImageSource::Render(render))
}
