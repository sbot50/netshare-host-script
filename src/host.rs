use std::collections::HashMap;
use std::thread::spawn;
use crate::audio_gui;
use crate::client::Client;

pub struct Host {
    clients: Vec<Client>,
    audio: Option<String>
}

impl Host {
    pub fn new() -> Host {
        Host {
            clients: Vec::new(),
            audio: None
        }
    }
    
    pub fn add_client(&mut self, id: String, nickname: String) {
        self.clients.push(Client::new(id, nickname));
    }

    pub fn set_controls(&mut self, id: String, controls: HashMap<String, f32>) {
        for i in 0..self.clients.len() {
            if self.clients[i].get_id() == id {
                self.clients[i].set(controls);
                break;
            }
        }
    }

    pub fn remove_client(&mut self, id: String) {
        for i in 0..self.clients.len() {
            if self.clients[i].get_id() == id {
                self.clients.remove(i);
                break;
            }
        }
    }
    
    pub fn select_audio(&mut self) {
        // spawn(|| {
        //     if let Err(e) = audio_gui::run() {
        //         eprintln!("GUI error: {e}");
        //     }
        // });
    }
}