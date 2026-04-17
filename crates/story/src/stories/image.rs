use std::path::Path;

use crate::prelude::*;
use gpui_engram_ui::components::image::center_crop_square;

const BALCONY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/balcony.jpg");

pub struct ImageStory {
    circle_source: gpui::ImageSource,
}

impl ImageStory {
    fn new() -> Self {
        Self {
            circle_source: center_crop_square(BALCONY).expect("failed to load balcony.jpg"),
        }
    }
}

impl Render for ImageStory {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let img_path = Path::new(BALCONY);

        v_flex().gap(Spacing::Large.pixels()).child(example_group(
            "Shapes",
            vec![
                example(
                    "Default",
                    Image::new(img_path)
                        .width(px(200.0))
                        .height(px(112.0))
                        .into_any_element(),
                ),
                example(
                    "Rounded",
                    Image::new(img_path)
                        .width(px(200.0))
                        .height(px(112.0))
                        .rounded(Radius::Medium)
                        .into_any_element(),
                ),
                example(
                    "Circle (center-cropped)",
                    Image::new(self.circle_source.clone())
                        .size(px(120.0))
                        .rounded_full()
                        .into_any_element(),
                ),
                example(
                    "Grayscale",
                    Image::new(img_path)
                        .width(px(200.0))
                        .height(px(112.0))
                        .grayscale(true)
                        .into_any_element(),
                ),
            ],
        ))
    }
}

pub fn build(_window: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_cx| ImageStory::new()).into()
}
