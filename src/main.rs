mod parse;
mod host;
mod client;

#[cfg(target_os = "linux")]
mod device_linux;

#[cfg(target_os = "windows")]
mod device_windows;
mod audio_gui;
mod audio_sink;

use std::net::TcpListener;
use std::sync::{mpsc, Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::spawn;
use serde::Serialize;
use tungstenite::{accept};
use crate::audio_gui::Source;
use crate::audio_sink::NullSinkGuard;

#[derive(Clone, Debug)]
enum ToGui {
    OpenPicker,
    WindowOpened(iced::window::Id),
    SourcesLoaded(Vec<Source>),
    SelectionChanged(Source),
    Submit,
}

#[derive(Clone, Debug)]
enum FromGui {
    Cancelled,
    Selected(Source),
}

fn main() -> iced::Result {
    let _sink = NullSinkGuard::new().expect("failed to create null sink");

    let (to_gui_tx, to_gui_rx) = mpsc::channel::<ToGui>();
    let (from_gui_tx, from_gui_rx) = mpsc::channel::<FromGui>();
    let from_gui_rx_shared = Arc::new(Mutex::new(from_gui_rx));
    spawn(move || {
        websocket(to_gui_tx, from_gui_rx_shared);
    });
    let from_gui_tx = Arc::new(Mutex::new(Some(from_gui_tx)));
    let to_gui_rx = Arc::new(Mutex::new(Some(to_gui_rx)));
    iced::daemon(
        move || {
            let tx = from_gui_tx.lock().unwrap().take().unwrap();
            let rx = to_gui_rx.lock().unwrap().take().unwrap();
            audio_gui::default(tx, rx)
        },
        audio_gui::update,
        audio_gui::view,
    )
        .theme(audio_gui::theme)
        .subscription(audio_gui::receiver_subscription)
        .run()
}

fn get_default_sink_monitor() -> String {
    let output = std::process::Command::new("pactl")
        .args(["info"])
        .output();

    if let Ok(output) = output {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            if let Some(sink_name) = line.strip_prefix("Default Sink:") {
                return format!("node.name={}.monitor", sink_name.trim());
            }
        }
    }

    "media.class=Audio/Source/Virtual".to_string()
}

fn websocket(send: Sender<ToGui>, receive: Arc<Mutex<Receiver<FromGui>>>) {
    let server = TcpListener::bind("127.0.0.1:6731").unwrap();
    for stream in server.incoming() {
        let send = send.clone();
        let receive = receive.clone();
        let stream = stream.unwrap();

        let mut websocket_conn = accept(stream).unwrap();
        let mut parser = parse::Parser::new();

        spawn(move || {
            loop {
                if let Ok(msg) = receive.lock().unwrap().recv() {
                    if let FromGui::Selected(stream_name) = msg {
                        let capture_props = if stream_name.name == "Entire System" {
                            format!("{}", stream_name.node_name)
                        } else {
                            format!("{}", stream_name.node_name)
                        };

                        std::process::Command::new("pw-loopback")
                            .args([
                                "--capture-props", &format!("target.object={}", stream_name.node_name),
                                "--playback-props", "target.object=netshare_sink",
                            ])
                            .spawn()
                            .ok();
                    }
                }
            }
        });

        loop {
            let msg_result = websocket_conn.read();
            match msg_result {
                Ok(msg) => {
                    if let Ok(text) = msg.to_text() {
                        parser.parse(text, &send);
                    }
                },
                Err(_) => { break; }
            }
        }
    }
}