use std::sync::Arc;

use gpui::*;
use gpui_component::{ActiveTheme, IconName, Sizable, Size as ComponentSize, spinner::Spinner};

use crate::utils::ui::{scale_size, viewport_image_size};

const MIN_ZOOM: f32 = 0.05;
const MAX_ZOOM: f32 = 32.0;

pub struct ImageViewer {
    state: Entity<ImageViewerState>,
    image: Option<Arc<Image>>,
}

#[derive(Debug)]
struct ImageViewerState {
    zoom: f32,
    pan: Point<Pixels>,
    viewport_origin: Point<Pixels>,
    viewport_size: Size<Pixels>,
    image_size: Size<Pixels>,
    fitted_once: bool,
    is_dragging: bool,
    drag_start_mouse: Option<Point<Pixels>>,
    drag_start_pan: Point<Pixels>,
}

#[derive(Clone, Copy)]
struct ViewportSnapshot {
    zoom: f32,
    pan: Point<Pixels>,
}

impl ImageViewerState {
    fn new() -> Self {
        Self {
            zoom: 1.0,
            pan: point(px(0.0), px(0.0)),
            viewport_origin: point(px(0.0), px(0.0)),
            viewport_size: size(px(0.0), px(0.0)),
            image_size: size(px(0.0), px(0.0)),
            fitted_once: false,
            is_dragging: false,
            drag_start_mouse: None,
            drag_start_pan: point(px(0.0), px(0.0)),
        }
    }

    fn update_viewport(&mut self, bounds: Bounds<Pixels>, image_size: Size<Pixels>) {
        self.viewport_origin = bounds.origin;
        self.viewport_size = bounds.size;
        self.image_size = image_size;
        self.fit_once(bounds.size, image_size);
    }

    fn fit_once(&mut self, viewport_size: Size<Pixels>, image_size: Size<Pixels>) {
        if self.fitted_once {
            return;
        }

        self.fit(viewport_size, image_size);
        self.fitted_once = true;
    }

    fn fit(&mut self, viewport_size: Size<Pixels>, image_size: Size<Pixels>) {
        if image_size.width.as_f32() == 0.0 || image_size.height.as_f32() == 0.0 {
            return;
        }

        let fit_zoom = (viewport_size.width.as_f32() / image_size.width.as_f32())
            .min(viewport_size.height.as_f32() / image_size.height.as_f32())
            .min(1.0)
            .clamp(MIN_ZOOM, MAX_ZOOM);
        let scaled_size = scale_size(image_size, fit_zoom);

        self.zoom = fit_zoom;
        self.pan = point(
            (viewport_size.width - scaled_size.width) / 2.0,
            (viewport_size.height - scaled_size.height) / 2.0,
        );
    }

    fn fit_to_viewport(&mut self) {
        self.fit(self.viewport_size, self.image_size);
    }

    fn start_drag_at(&mut self, position: &Point<Pixels>) {
        self.is_dragging = true;
        self.drag_start_mouse = Some(*position);
        self.drag_start_pan = self.pan;
    }

    fn pan_by(&mut self, delta: Point<Pixels>) {
        self.pan += delta;
    }

    fn zoom_around(&mut self, window_position: Point<Pixels>, factor: f32) {
        let position = window_position - self.viewport_origin;
        let old_zoom = self.zoom;
        self.zoom = (self.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);

        if self.zoom == old_zoom {
            return;
        }

        let ratio = self.zoom / old_zoom;
        self.pan = position - (position - self.pan) * ratio;
    }

    fn reanchor_drag(&mut self, position: Point<Pixels>) {
        if self.is_dragging {
            self.drag_start_mouse = Some(position);
            self.drag_start_pan = self.pan;
        }
    }

    fn snapshot(&self) -> ViewportSnapshot {
        ViewportSnapshot {
            zoom: self.zoom,
            pan: self.pan,
        }
    }
}

impl ImageViewer {
    pub fn new(cx: &mut Context<Self>, image: Option<Image>) -> Self {
        Self {
            state: cx.new(|_| ImageViewerState::new()),
            image: image.and_then(|image| Some(Arc::new(image))).or(None),
        }
    }
}

impl Render for ImageViewer {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.clone();
        let zoom_label = format!("{:.0}%", self.state.read(cx).zoom * 100.0);
        let colors = cx.theme().colors;

        let Some(image) = &self.image else {
            return div().size_full();
        };

        let Some(image_render) = image.clone().use_render_image(window, cx) else {
            return div()
                .size_full()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_2()
                .text_color(colors.muted_foreground)
                .child(
                    Spinner::new()
                        .with_size(ComponentSize::Large)
                        .color(colors.muted_foreground)
                        .icon(IconName::LoaderCircle),
                )
                .child(div().child("Loading image..."));
        };

        let cursor = if self.state.read(cx).is_dragging {
            CursorStyle::ClosedHand
        } else {
            CursorStyle::OpenHand
        };

        div()
            .cursor(cursor)
            .relative()
            .size_full()
            .rounded_sm()
            .overflow_hidden()
            .on_any_mouse_down(cx.listener(|this, event: &MouseDownEvent, _window, cx| {
                this.state.update(cx, |state, cx| match event.button {
                    MouseButton::Left => {
                        if event.click_count == 2 {
                            state.fit_to_viewport();
                            cx.notify();
                        } else if !state.is_dragging {
                            state.start_drag_at(&event.position);
                            cx.notify();
                        }
                    }
                    MouseButton::Right => {}
                    MouseButton::Middle => {}
                    MouseButton::Navigate(_direction) => {}
                });
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _event: &MouseUpEvent, _window, cx| {
                    this.state.update(cx, |state, cx| {
                        if state.is_dragging {
                            state.is_dragging = false;
                        }

                        state.drag_start_mouse = None;
                        cx.notify();
                    });
                }),
            )
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _window, cx| {
                this.state.update(cx, |state, cx| {
                    if event.dragging()
                        && event.pressed_button.is_some_and(|b| b == MouseButton::Left)
                    {
                        let Some(drag_start_mouse) = state.drag_start_mouse else {
                            return;
                        };

                        state.pan = state.drag_start_pan + (event.position - drag_start_mouse);
                        cx.notify();
                    }
                })
            }))
            .on_pinch(cx.listener(|this, event: &PinchEvent, _window, cx| {
                let factor = (1.0 + event.delta).max(0.01);
                this.state.update(cx, |state, cx| {
                    state.zoom_around(event.position, factor);
                    state.reanchor_drag(event.position);
                    cx.notify();
                });
            }))
            .on_scroll_wheel(cx.listener(|this, event: &ScrollWheelEvent, _window, cx| {
                let delta = match event.delta {
                    ScrollDelta::Pixels(delta) => delta,
                    ScrollDelta::Lines(delta) => point(px(delta.x * 16.0), px(delta.y * 16.0)),
                };

                this.state.update(cx, |state, cx| {
                    let factor = (delta.y.as_f32() * 0.0015).exp();
                    state.zoom_around(event.position, factor);
                    state.reanchor_drag(event.position);
                    cx.notify();
                });
            }))
            .child(
                canvas(
                    {
                        let state = state.clone();
                        let image_render = image_render.clone();
                        move |bounds, window, cx| {
                            let image_size = viewport_image_size(&image_render, window);

                            state.update(cx, |state, _cx| {
                                state.update_viewport(bounds, image_size);
                                state.snapshot()
                            })
                        }
                    },
                    move |bounds, snapshot, window, _cx| {
                        let image_size = viewport_image_size(&image_render, window);
                        let image_bounds = gpui::bounds(
                            bounds.origin + snapshot.pan,
                            scale_size(image_size, snapshot.zoom),
                        );

                        window
                            .paint_image(
                                image_bounds,
                                Corners::all(px(0.0)),
                                image_render,
                                0,
                                false,
                            )
                            .ok();
                    },
                )
                .size_full(),
            )
            .child(
                div()
                    .absolute()
                    .left(px(8.0))
                    .bottom(px(8.0))
                    .px(px(8.0))
                    .py(px(4.0))
                    .rounded_sm()
                    .bg(colors.secondary.alpha(0.65))
                    .border_1()
                    .border_color(colors.border)
                    .text_color(colors.primary)
                    .text_xs()
                    .child(zoom_label),
            )
    }
}
