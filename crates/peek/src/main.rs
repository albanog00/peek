use gpui::{
    AppContext, Context, Entity, IntoElement, ParentElement, Render, Styled, TitlebarOptions,
    Window, WindowOptions, div,
};
use gpui_component::{
    Root, StyledExt,
    button::{Button, ButtonVariants},
};

pub struct Counter {
    value: u64,
}

impl Counter {
    pub fn new() -> Self {
        Self { value: 0 }
    }
}

impl Render for Counter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .v_flex()
            .gap_2()
            .items_center()
            .justify_center()
            .child(format!("{}", self.value))
            .child(
                Button::new("increment")
                    .primary()
                    .label("Increment")
                    .on_click(cx.listener(|this, _, _, _| this.value += 1)),
            )
    }
}

pub struct HelloWorld {
    counter: Entity<Counter>,
}

impl HelloWorld {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            counter: cx.new(|_| Counter::new()),
        }
    }
}

impl Render for HelloWorld {
    fn render(&mut self, _: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .v_flex()
            .gap_2()
            .size_full()
            .items_center()
            .justify_center()
            .child(self.counter.clone())
    }
}

fn main() {
    let app = gpui_platform::application().with_assets(gpui_component_assets::Assets);

    app.run(move |cx| {
        gpui_component::init(cx);

        let window_opts = WindowOptions {
            titlebar: Some(TitlebarOptions {
                title: Some("Peek | Image viewer".into()),
                ..Default::default()
            }),
            ..Default::default()
        };

        cx.spawn(async move |cx| {
            cx.open_window(window_opts, |window, cx| {
                let view = cx.new(|cx| HelloWorld::new(cx));
                cx.new(|cx| Root::new(view, window, cx))
            })
            .expect("Failed to open window");
        })
        .detach();
    });
}
