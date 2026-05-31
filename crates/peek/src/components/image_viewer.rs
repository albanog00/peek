use std::sync::Arc;

use gpui::{prelude::FluentBuilder, *};
use gpui_component::{ActiveTheme, IconName, Sizable, Size as ComponentSize, spinner::Spinner};

use crate::utils::ui::{scale_size, size_is_zero, viewport_image_size};

const MIN_ZOOM: f32 = 0.05;
const MAX_ZOOM: f32 = 32.0;
const DOUBLE_CLICK_ZOOM_FACTOR: f32 = 4.0;

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
    has_initial_fit: bool,
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
            has_initial_fit: false,
            is_dragging: false,
            drag_start_mouse: None,
            drag_start_pan: point(px(0.0), px(0.0)),
        }
    }

    fn set_viewport_metrics(&mut self, bounds: Bounds<Pixels>, image_size: Size<Pixels>) {
        self.viewport_origin = bounds.origin;
        self.viewport_size = bounds.size;
        self.image_size = image_size;
    }

    fn fit(&mut self, viewport_size: Size<Pixels>, image_size: Size<Pixels>) -> bool {
        if size_is_zero(viewport_size) || size_is_zero(image_size) {
            return false;
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
        true
    }

    fn fit_to_viewport(&mut self) -> bool {
        let fitted = self.fit(self.viewport_size, self.image_size);
        self.has_initial_fit |= fitted;
        fitted
    }

    fn is_viewport_fit(&self) -> bool {
        if size_is_zero(self.viewport_size) || size_is_zero(self.image_size) {
            return false;
        }

        let fit_zoom = (self.viewport_size.width.as_f32() / self.image_size.width.as_f32())
            .min(self.viewport_size.height.as_f32() / self.image_size.height.as_f32())
            .min(1.0)
            .clamp(MIN_ZOOM, MAX_ZOOM);

        (fit_zoom - self.zoom).abs() < 0.001
    }

    fn start_drag_at(&mut self, position: &Point<Pixels>) {
        self.is_dragging = true;
        self.drag_start_mouse = Some(*position);
        self.drag_start_pan = self.pan;
    }

    fn stop_drag(&mut self) -> bool {
        if !self.is_dragging {
            return false;
        }

        self.is_dragging = false;
        self.drag_start_mouse = None;
        true
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

    fn handle_double_click(&mut self, position: Point<Pixels>) {
        self.stop_drag();
        if self.is_viewport_fit() {
            self.zoom_around(position, DOUBLE_CLICK_ZOOM_FACTOR);
        } else {
            self.fit_to_viewport();
        }
    }

    fn zoom_label(&self) -> Option<SharedString> {
        self.has_initial_fit
            .then(|| format!("{:.0}%", self.zoom * 100.0).into())
    }
}

impl ImageViewer {
    pub fn new(cx: &mut Context<Self>, image: Option<Image>) -> Self {
        Self {
            state: cx.new(|_| ImageViewerState::new()),
            image: image.map(Arc::new),
        }
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) {
        self.state.update(cx, |state, cx| match event.button {
            MouseButton::Left => {
                if event.click_count % 2 == 0 {
                    state.handle_double_click(event.position);
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
    }

    fn on_mouse_up(
        &mut self,
        _event: &MouseUpEvent,
        _window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) {
        self.state.update(cx, |state, cx| {
            if state.stop_drag() {
                cx.notify();
            }
        });
    }

    fn on_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        _window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) {
        self.state.update(cx, |state, cx| {
            if !event.dragging() || event.pressed_button != Some(MouseButton::Left) {
                if state.stop_drag() {
                    cx.notify();
                }
                return;
            }

            let Some(drag_start_mouse) = state.drag_start_mouse else {
                return;
            };

            state.pan = state.drag_start_pan + (event.position - drag_start_mouse);
            cx.notify();
        })
    }

    fn on_pinch(&mut self, event: &PinchEvent, _window: &mut Window, cx: &mut Context<'_, Self>) {
        let factor = (1.0 + event.delta).max(0.01);
        self.state.update(cx, |state, cx| {
            state.zoom_around(event.position, factor);
            state.reanchor_drag(event.position);
            cx.notify();
        });
    }

    fn on_scroll(
        &mut self,
        event: &ScrollWheelEvent,
        _window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) {
        let delta = match event.delta {
            ScrollDelta::Pixels(delta) => delta,
            ScrollDelta::Lines(delta) => point(px(delta.x * 16.0), px(delta.y * 16.0)),
        };

        self.state.update(cx, |state, cx| {
            let factor = (delta.y.as_f32() * 0.0015).exp();
            state.zoom_around(event.position, factor);
            state.reanchor_drag(event.position);
            cx.notify();
        });
    }
}

impl Render for ImageViewer {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.clone();
        let zoom_label = state.read(cx).zoom_label();
        let colors = cx.theme().colors;

        let container = div().size_full().relative().overflow_hidden();

        let Some(image) = self.image.clone() else {
            // No Image provided
            return container
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_2()
                .text_color(colors.muted_foreground)
                .child("Drop an image here!");
        };

        let Some(image_render) = image.use_render_image(window, cx) else {
            // Image is loading
            return container
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

        container
            .cursor(cursor)
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .on_pinch(cx.listener(Self::on_pinch))
            .on_scroll_wheel(cx.listener(Self::on_scroll))
            .child(
                canvas(
                    {
                        let state = state.clone();
                        let image_render = image_render.clone();
                        move |bounds, window, app| {
                            let image_size = viewport_image_size(&image_render, window);

                            state.update(app, |state, cx| {
                                state.set_viewport_metrics(bounds, image_size);
                                if !state.has_initial_fit && state.fit_to_viewport() {
                                    cx.on_next_frame(window, move |_, _, cx| {
                                        cx.notify();
                                    });
                                }
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
            .when_some(zoom_label, move |el, zoom_label| {
                el.child(
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
            })
    }
}
