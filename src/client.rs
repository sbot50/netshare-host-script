use std::collections::HashMap;
use crate::device::Device;

pub struct Client {
    id: u16,
    device: Device
}

impl Client {
    pub fn new(id: u16, nickname: String) -> Client {
        Client {
            id,
            device: Device::new(format!("{}'s Controller", nickname).as_str())
        }
    }
    
    pub fn get_id(&self) -> u16 {
        self.id
    }
    
    pub fn set(&mut self, controls: HashMap<String, f32>) {
        self.device.set_controls(controls);
    }
}