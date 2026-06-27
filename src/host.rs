use std::collections::HashMap;
use crate::client::Client;

pub struct Host {
    clients: Vec<Client>
}

impl Host {
    pub fn new() -> Host {
        Host {
            clients: Vec::new()
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
}