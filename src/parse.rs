use std::collections::HashMap;
use std::sync::mpsc::Sender;
use serde::{Deserialize};
use serde_json::{Value};
use crate::host::Host;
use crate::ToGui;

#[derive(Deserialize)]
#[allow(dead_code)]
struct Connect {
    rtype: String,
    id: String,
    nickname: String
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct Data {
    rtype: String,
    id: String,
    data: HashMap<String, f32>
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct Disconnect {
    rtype: String,
    id: String
}

pub struct Parser {
    host: Host,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            host: Host::new(),
        }
    }

    pub fn parse(&mut self, json: &str, send: &Sender<ToGui>) {
        let data = serde_json::from_str(&json);
        if data.is_err() {
            return;
        }
        let data: Value = data.unwrap();
        let rtype = match data["rtype"].as_str() {
            Some(rtype) => rtype,
            None => return,
        };
        match rtype {
            "connect" => {
                let data: Connect = serde_json::from_str(&json).unwrap();
                self.host.add_client(data.id, data.nickname);
            },
            "controls" => {
                let data: Data = serde_json::from_str(&json).unwrap();
                self.host.set_controls(data.id, data.data);
            },
            "disconnect" => {
                let data: Disconnect = serde_json::from_str(&json).unwrap();
                self.host.remove_client(data.id);
            },
            "get_audio" => {
                send.send(ToGui::OpenPicker).unwrap();
            },
            _ => return,
        }
    }
}