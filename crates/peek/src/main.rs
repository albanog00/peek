use gpui::*;
use gpui_component::*;

mod components;
mod utils;

use components::image_viewer::ImageViewer;

use crate::utils::fs::{LoadImageError, load_image_from_path};

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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors;
        let notification_layer = Root::render_notification_layer(window, cx);

        div()
            .bg(colors.background)
            .size_full()
            .can_drop(|_, _, _| true)
            .on_drop(cx.listener(
                |this, paths: &ExternalPaths, window: &mut Window, cx: &mut Context<'_, Self>| {
                    if let Some(path) = paths.paths().first() {
                        let result = utils::fs::load_image_from_path(path);
                        this.viewer.update(cx, |viewer, cx| match result {
                            Ok(image) => viewer.set_image(image, cx),
                            Err(error) => {
                                tracing::error!(%error);
                                window.push_notification(error.to_string(), cx);
                            }
                        });
                    }
                },
            ))
            .children(notification_layer)
            .child(self.viewer.clone())
    }
}

fn load_initial_image(path: Option<&String>) -> (Option<Image>, Option<LoadImageError>) {
    match path {
        Some(path) => match load_image_from_path(path) {
            Ok(image) => (Some(image), None),
            Err(error) => (None, Some(error)),
        },
        None => (None, None),
    }
}

fn main() {
    tracing_subscriber::fmt().init();

    let args = std::env::args().collect::<Vec<_>>();

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

                    let (image, error) = load_initial_image(args.get(1));
                    let root = cx
                        .new(|cx| Root::new(cx.new(|cx| MainContent::new(cx, image)), window, cx));

                    if let Some(error) = error {
                        root.update(cx, |root, cx| {
                            tracing::error!(%error);
                            root.push_notification(error.to_string(), window, cx);
                        });
                    }

                    root
                })
                .expect("Failed to open window");
            })
            .detach();
        });
}
