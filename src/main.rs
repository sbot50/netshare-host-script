mod parse;
mod host;
mod client;

#[cfg(target_os = "linux")]
mod device_linux;

#[cfg(target_os = "windows")]
mod device_windows;
mod audio_gui;

use std::net::TcpListener;
use std::sync::{mpsc, Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::spawn;
use serde::Serialize;
use tungstenite::{accept, Utf8Bytes};

#[derive(Serialize)]
struct AudioStreamResponse {
    rtype: String,
    stream: String,
}

#[derive(Clone, Debug)]
enum ToGui {
    OpenPicker,
    WindowOpened(iced::window::Id),
    SourcesLoaded(Vec<String>),
    SelectionChanged(String),
    Submit,
}

#[derive(Clone, Debug)]
enum FromGui {
    Cancelled,
    Selected(String),
}

fn main() -> iced::Result {
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

fn websocket(send: Sender<ToGui>, receive: Arc<Mutex<Receiver<FromGui>>>) {
    let server = TcpListener::bind("127.0.0.1:6731").unwrap();
    for stream in server.incoming() {
        let send = send.clone();
        let receive = receive.clone();
        let stream = stream.unwrap();

        let websocket_conn = Arc::new(Mutex::new(accept(stream).unwrap()));
        let websocket_conn_clone = websocket_conn.clone();
        let mut parser = parse::Parser::new(); // Initialize once per connection

        // Thread to forward FromGui messages back over the websocket
        spawn(move || {
            loop {
                if let Ok(msg) = receive.lock().unwrap().recv() {
                    println!("{:?}", msg);
                    if let FromGui::Selected(stream_name) = msg {
                        let response = AudioStreamResponse {
                            rtype: "selected_audio".into(),
                            stream: stream_name,
                        };
                        let json = serde_json::to_string(&response).unwrap();

                        // Fix 1: Use a string reference to match Utf8Bytes expectations
                        if let Ok(mut ws) = websocket_conn_clone.lock() {
                            let _ = ws.send(tungstenite::Message::Text(Utf8Bytes::from(&json)));
                        }
                    }
                }
            }
        });

        // Main reading loop
        loop {
            let msg_result = websocket_conn.lock().unwrap().read();
            match msg_result {
                // Fix 2: Use if let to safely handle ownership/moves without .unwrap()
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