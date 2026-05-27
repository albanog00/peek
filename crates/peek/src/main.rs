use iced::{
    Element,
    widget::{button, column, text},
};

#[derive(Clone, Debug)]
enum Message {
    Increment,
}

#[derive(Clone, Debug, Default)]
struct Counter {
    value: u64,
}

fn update(counter: &mut Counter, message: Message) {
    match message {
        Message::Increment => counter.value += 1,
    }
}

fn view(counter: &Counter) -> Element<'_, Message> {
    column![
        text(counter.value).size(20),
        button("Increment").on_press(Message::Increment),
    ]
    .spacing(10)
    .into()
}

fn main() -> iced::Result {
    iced::run(update, view)
}
