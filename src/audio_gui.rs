use iced::{Element, Task};

pub fn run() -> iced::Result {
    iced::application(default, update, view)
        .run()
}

struct AudioGui;

fn default() -> AudioGui {
    AudioGui
}

fn update(_state: &mut AudioGui, _message: ()) -> Task<()> {
    Task::none()
}

fn view(_state: &'_ AudioGui) -> Element<'_, ()> {
    "Choose audio device".into()
}