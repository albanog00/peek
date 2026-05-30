use gpui::*;

pub fn viewport_image_size(image: &RenderImage, window: &Window) -> Size<Pixels> {
    image
        .size(0)
        .map(|value| px(value.0 as f32 / window.scale_factor()))
}

pub fn scale_size(size: Size<Pixels>, factor: f32) -> Size<Pixels> {
    gpui::size(
        px(size.width.as_f32() * factor),
        px(size.height.as_f32() * factor),
    )
}
