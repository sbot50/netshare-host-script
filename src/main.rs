mod parse;
mod host;
mod client;

#[cfg(target_os = "linux")]
mod device_linux;

#[cfg(target_os = "windows")]
mod device_windows;
#[cfg(target_os = "linux")]
mod audio_gui;
#[cfg(target_os = "linux")]
mod audio_sink;

use std::net::TcpListener;
use std::sync::{mpsc, Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::spawn;
use tungstenite::{accept};
#[cfg(target_os = "linux")]
use crate::audio_gui::Source;
#[cfg(target_os = "linux")]
use crate::audio_sink::NullSinkGuard;

#[derive(Clone, Debug)]
enum ToGui {
    OpenPicker,
    WindowOpened(iced::window::Id),
    #[cfg(target_os = "linux")]
    SourcesLoaded(Vec<Source>),
    #[cfg(target_os = "linux")]
    SelectionChanged(Source),
    Submit,
}

#[derive(Clone, Debug)]
enum FromGui {
    Cancelled,
    #[cfg(target_os = "linux")]
    Selected(Source),
}

fn main() -> iced::Result {
    #[cfg(target_os = "linux")]
    {
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
    #[cfg(target_os = "windows")]
    {
        let (to_gui_tx, _) = mpsc::channel::<ToGui>();
        let (_, from_gui_rx) = mpsc::channel::<FromGui>();
        let from_gui_rx_shared = Arc::new(Mutex::new(from_gui_rx));
        websocket(to_gui_tx, from_gui_rx_shared);
    }
}

fn websocket(send: Sender<ToGui>, receive: Arc<Mutex<Receiver<FromGui>>>) {
    let server = TcpListener::bind("127.0.0.1:6731").unwrap();
    for stream in server.incoming() {
        let send = send.clone();
        let _receive = receive.clone();
        let stream = stream.unwrap();

        let mut websocket_conn = accept(stream).unwrap();
        let mut parser = parse::Parser::new();

        #[cfg(target_os = "linux")]
        spawn(move || {
            let mut active_loopback_id: Option<String> = None;

            loop {
                if let Ok(msg) = _receive.lock().unwrap().recv() {
                    if let FromGui::Selected(stream_name) = msg {
                        if let Some(id) = active_loopback_id.take() {
                            let _ = std::process::Command::new("pactl")
                                .args(["unload-module", &id])
                                .output();
                        }

                        let output = std::process::Command::new("pactl")
                            .args([
                                "load-module", "module-loopback",
                                &format!("source={}", stream_name.node_name),
                                "sink=netshare_sink",
                                "latency_msec=1",
                            ])
                            .output();

                        if let Ok(out) = output {
                            if out.status.success() {
                                let new_id = String::from_utf8_lossy(&out.stdout).trim().to_string();
                                active_loopback_id = Some(new_id);
                            }
                        }
                    } else if let FromGui::Cancelled = msg {
                        if let Some(id) = active_loopback_id.take() {
                            let _ = std::process::Command::new("pactl")
                                .args(["unload-module", &id])
                                .output();
                        }
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