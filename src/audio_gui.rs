use std::cmp::Ordering;
use std::sync::{Arc, Mutex, OnceLock};
use std::sync::mpsc::{Receiver, Sender};
use iced::{stream, Element, Subscription, Task, Theme};
use iced::futures::SinkExt;
use iced::widget::{button, column, pick_list, text};
use crate::{FromGui, ToGui};

static RECEIVER: OnceLock<Arc<Mutex<Receiver<ToGui>>>> = OnceLock::new();

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd)]
pub struct Source {
    pub name: String,
    pub node_name: String,
}

impl Ord for Source {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub struct AudioGui {
    pub sender: Sender<FromGui>,
    sources: Vec<Source>,
    selected: Option<Source>,
    window_id: iced::window::Id,
}

pub fn theme(_state: &AudioGui, _window: iced::window::Id) -> Theme {
    Theme::Dark
}

pub fn default(sender: Sender<FromGui>, receiver: Receiver<ToGui>) -> (AudioGui, Task<ToGui>) {
    RECEIVER.get_or_init(|| Arc::new(Mutex::new(receiver)));
    (AudioGui { sender, sources: Vec::new(), selected: None, window_id: iced::window::Id::unique() }, Task::none())}

pub fn update(state: &mut AudioGui, message: ToGui) -> Task<ToGui> {
    match message {
        ToGui::OpenPicker => {
            let window_settings = iced::window::Settings {
                size: iced::Size::new(350.0, 200.0), // A safe starting minimum guess
                ..Default::default()
            };
            let (_id, task) = iced::window::open(window_settings);
            state.window_id = _id;
            task.map(ToGui::WindowOpened)
        }
        ToGui::WindowOpened(_id) => {
            Task::done(ToGui::SourcesLoaded(load_audio_sources()))
        }
        ToGui::SourcesLoaded(sources) => {
            state.sources = sources;
            state.selected = state.sources.first().cloned();
            Task::none()
        }
        ToGui::SelectionChanged(source) => {
            state.selected = Some(source);
            Task::none()
        }
        ToGui::Submit => {
            match &state.selected {
                Some(s) => state.sender.send(FromGui::Selected(s.clone())).ok(),
                None => state.sender.send(FromGui::Cancelled).ok(),
            };
            iced::window::close(state.window_id)
        }
    }
}

pub fn view(state: &'_ AudioGui, _window: iced::window::Id) -> Element<'_, ToGui> {
    column![
        text("Choose audio stream or device"),
        pick_list(
            state.sources.as_slice(),
            state.selected.as_ref(),
            ToGui::SelectionChanged,
        ),
        button("Submit").on_press(ToGui::Submit),
    ]
        .spacing(10)
        .padding(20)
        .into()
}

fn load_audio_sources() -> Vec<Source> {
    let mut sources = Vec::new();
    let mut entire_system = Vec::new();

    if let Ok(out) = std::process::Command::new("pactl")
        .args(["list", "short", "sinks"])
        .output()
    {
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();

            if parts.len() >= 2 && parts[1] != "netshare_sink" {
                entire_system.push(Source {
                    name: "Entire System".to_string(),
                    node_name: format!("{}.monitor", parts[1]),
                });
            }
        }
    }

    if let Ok(out) = std::process::Command::new("pactl")
        .args(["list", "sink-inputs"])
        .output()
    {
        let stdout_str = String::from_utf8_lossy(&out.stdout);

        let mut current_id = None;
        let mut current_name = None;

        for line in stdout_str.lines() {
            let line = line.trim();

            if line.starts_with("Sink Input #") {
                if let (Some(id), Some(name)) =
                    (current_id.take(), current_name.take())
                {
                    sources.push(Source {
                        name,
                        node_name: id,
                    });
                }

                current_id = line
                    .strip_prefix("Sink Input #")
                    .map(|s| s.to_string());

                continue;
            }

            if line.starts_with("application.name = \"") {
                if let Some(name) = line.split('"').nth(1) {
                    current_name = Some(name.to_string());
                }
            } else if current_name.is_none()
                && line.starts_with("media.name = \"")
            {
                if let Some(name) = line.split('"').nth(1) {
                    current_name = Some(name.to_string());
                }
            }
        }

        if let (Some(id), Some(name)) = (current_id, current_name) {
            sources.push(Source {
                name,
                node_name: id,
            });
        }
    }

    sources.splice(0..0, entire_system);

    sources
}

pub fn receiver_stream() -> impl iced::futures::Stream<Item = ToGui> {
    let receiver = RECEIVER.get().unwrap().clone();
    stream::channel(10, async move |mut output| {
        loop {
            let msg = tokio::task::spawn_blocking({
                let receiver = receiver.clone();
                move || receiver.lock().unwrap().recv()
            }).await;
            if let Ok(Ok(msg)) = msg {
                output.send(msg).await.ok();
            }
        }
    })
}

pub fn receiver_subscription(_state: &AudioGui) -> Subscription<ToGui> {
    Subscription::run(receiver_stream)
}