use gpui::*;
use gpui_component::*;

mod components;
mod utils;

use components::image_viewer::ImageViewer;

use crate::utils::fs::load_image_from_path;

pub struct MainContent {
    viewer: Entity<ImageViewer>,
}

impl MainContent {
    pub fn new(cx: &mut Context<Self>, image: Option<Image>) -> Self {
        Self {
            viewer: cx.new(|cx| ImageViewer::new(cx, image)),
        }
    }
}

impl Render for MainContent {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors;
        div()
            .bg(colors.background)
            .size_full()
            .child(self.viewer.clone())
    }
}

fn main() {
    tracing_subscriber::fmt().init();

    let args = std::env::args().collect::<Vec<_>>();
    let input_provided_image = args.get(1).and_then(load_image_from_path);

    gpui_platform::application()
        .with_assets(gpui_component_assets::Assets)
        .run(move |cx| {
            gpui_component::init(cx);

            let window_opts = WindowOptions {
                titlebar: Some(TitlebarOptions {
                    title: Some("Peek | Image viewer".into()),
                    ..Default::default()
                }),
                window_bounds: Some(WindowBounds::Windowed(bounds(
                    point(px(0.0), px(0.0)),
                    size(px(1024.0), px(768.0)),
                ))),
                window_min_size: Some(size(px(250.0), px(250.0))),
                kind: WindowKind::Normal,
                ..Default::default()
            };

            cx.spawn(async move |cx| {
                cx.open_window(window_opts, |window, cx| {
                    window
                        .observe_window_appearance(|window, cx| {
                            theme::Theme::sync_system_appearance(Some(window), cx);
                        })
                        .detach();

                    let view = cx.new(|cx| MainContent::new(cx, input_provided_image));
                    cx.new(|cx| Root::new(view, window, cx))
                })
                .expect("Failed to open window");
            })
            .detach();
        });
}
