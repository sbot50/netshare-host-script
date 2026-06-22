mod parse;
mod host;
mod client;

#[cfg(target_os = "linux")]
mod device_linux;

#[cfg(target_os = "windows")]
mod device_windows;
mod audio_gui;

use std::net::TcpListener;
use std::thread::spawn;
use tungstenite::accept;

fn main () {
    let server = TcpListener::bind("127.0.0.1:6731").unwrap();
    for stream in server.incoming() {
        spawn (move || {
            let mut parser = parse::Parser::new();
            let mut websocket = accept(stream.unwrap()).unwrap();
            loop {
                let msg_result = websocket.read();
                match msg_result {
                    Ok(msg) => {
                        if msg.is_text() {
                            let msg = msg.to_text().unwrap();
                            parser.parse(msg);
                        }
                    },
                    Err(_) => { break; }
                }
            }
        });
    }
}