use std::cmp::Ordering;
use std::sync::{Arc, Mutex, OnceLock};
use std::sync::mpsc::{Receiver, Sender};
use iced::{stream, Element, Subscription, Task, Theme};
use iced::futures::SinkExt;
use iced::widget::{button, column, pick_list, text};
use crate::{FromGui, ToGui};

use pipewire as pw;
use pipewire::context::Context;
use pipewire::main_loop::MainLoop;

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
            let (_id, task) = iced::window::open(iced::window::Settings::default());
            state.window_id = _id;
            task.map(ToGui::WindowOpened)
        }
        ToGui::WindowOpened(_id) => {
            Task::done(ToGui::SourcesLoaded(load_audio_sources()))
        }
        ToGui::SourcesLoaded(sources) => {
            state.sources = sources;
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
            let _: Task<ToGui> = iced::window::close(state.window_id);
            Task::none()
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
    pw::init();

    let mainloop = MainLoop::new(None).expect("Failed to create PipeWire MainLoop");
    let context = Context::new(&mainloop).expect("Failed to create PipeWire Context");
    let core = context.connect(None).expect("Failed to connect to PipeWire Core");
    let registry = core.get_registry().expect("Failed to get PipeWire Registry");

    let sources = Arc::new(Mutex::new(Vec::new()));

    let sources_clone = sources.clone();
    let _registry_listener = registry
        .add_listener_local()
        .global(move |global| {
            if global.type_ == pw::types::ObjectType::Node {
                if let Some(props) = global.props {
                    if let Some(media_class) = props.get("media.class") {
                        let is_app_stream = media_class == "Stream/Output/Audio";
                        let is_system_sink = media_class == "Audio/Sink";

                        if is_app_stream || is_system_sink {
                            let name = props.get("node.description")
                                .or_else(|| props.get("application.name"))
                                .or_else(|| props.get("node.name"))
                                .unwrap_or("Unknown Stream");

                            let formatted_name = if is_system_sink {
                                "Entire System".to_string()
                            } else {
                                name.split(".").nth(0).unwrap().to_string()
                            };

                            let source = Source {
                                name: formatted_name,
                                node_name: if is_system_sink {
                                    format!("\"{}.monitor\"", props.get("node.name").unwrap_or(""))
                                } else {
                                    format!("\"{}\"", props.get("node.name").unwrap_or(""))
                                }
                            };
                            sources_clone.lock().unwrap().push(source);
                        }
                    }
                }
            }
        })
        .register();

    let mainloop_clone = mainloop.clone();
    let _core_listener = core
        .add_listener_local()
        .done(move |_id, _seq| {
            mainloop_clone.quit();
        })
        .register();

    core.sync(0).ok();
    mainloop.run();

    let mut result = sources.lock().unwrap().clone();
    result.sort();
    result
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