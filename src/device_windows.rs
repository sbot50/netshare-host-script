use std::collections::HashMap;
use vigem_client::{Client, Xbox360Wired, XGamepad, XButtons, TargetId};

pub struct Device {
    device: Xbox360Wired<Client>,
    button_states: HashMap<u16, u16>,
    axis_states: HashMap<String, i16>,
}

impl Device {
    pub fn new(name: &str) -> Device {
        let client = Client::connect().unwrap();

        let mut target = Xbox360Wired::new(client, TargetId::default());
        target.plugin().unwrap();
        target.wait_ready().unwrap();

        Device {
            device: target,
            button_states: HashMap::new(),
            axis_states: HashMap::new(),
        }
    }

    pub fn set_controls(&mut self, controls: HashMap<String, f32>) {
        for (key, value) in controls {
            if let Some(key) = self.string_to_key(&key) {
                self.button_states.insert(key, value as u16);
            } else if let Some(axis) = self.string_to_axis(&key) {
                if key.contains("trigger") {
                    self.axis_states.insert(axis.to_string(), (value * 255.0) as i16);
                }
                else {
                    self.axis_states.insert(axis.to_string(), (value * 32767.0) as i16);
                }
            }
        }
        let mut gamepad = XGamepad {
            thumb_lx: *self.axis_states.get("left_thumb_x").unwrap_or(&0i16) * -1,
            thumb_ly: *self.axis_states.get("left_thumb_y").unwrap_or(&0i16),
            thumb_rx: *self.axis_states.get("right_thumb_x").unwrap_or(&0i16) * -1,
            thumb_ry: *self.axis_states.get("right_thumb_y").unwrap_or(&0i16),
            left_trigger: *self.axis_states.get("left_trigger").unwrap_or(&0i16) as u8,
            right_trigger: *self.axis_states.get("right_trigger").unwrap_or(&0i16) as u8,
            buttons: XButtons::default(),
        };
        let mut buttons: u16 = 0;
        for (key, value) in &self.button_states {
            if value > &0 {
                buttons |= key;
            }
        }
        gamepad.buttons = XButtons::from(buttons);
        self.device.update(&gamepad).unwrap();
    }

    fn string_to_key(&self, key: &str) -> Option<u16> {
        match key {
            "a" => Some(XButtons::A),
            "b" => Some(XButtons::B),
            "x" => Some(XButtons::X),
            "y" => Some(XButtons::Y),
            "l1" => Some(XButtons::LB),
            "r1" => Some(XButtons::RB),
            "l3" => Some(XButtons::LTHUMB),
            "r3" => Some(XButtons::RTHUMB),
            "minus" => Some(XButtons::BACK),
            "plus" => Some(XButtons::START),
            "up" => Some(XButtons::UP),
            "down" => Some(XButtons::DOWN),
            "left" => Some(XButtons::LEFT),
            "right" => Some(XButtons::RIGHT),
            _ => None
        }
    }

    fn string_to_axis(&self, axis: &str) -> Option<&str> {
        match axis {
            "left_y" => Some("left_thumb_y"),
            "left_x" => Some("left_thumb_x"),
            "right_y" => Some("right_thumb_y"),
            "right_x" => Some("right_thumb_x"),
            "left_trigger" => Some("left_trigger"),
            "right_trigger" => Some("right_trigger"),
            _ => None
        }
    }
}