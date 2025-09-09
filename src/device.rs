use std::collections::HashMap;
use evdev::{AbsInfo, AbsoluteAxisCode, AttributeSet, EventType, InputEvent, KeyCode, UinputAbsSetup};
use evdev::uinput::{ VirtualDevice };

struct Axis {
    code: AbsoluteAxisCode,
    value: i32,
    min: i32,
    max: i32,
    fuzz: i32,
    flat: i32,
    resolution: i32
}

impl Axis {
    pub const fn new(code: AbsoluteAxisCode, value: i32, min: i32, max: i32, fuzz: i32, flat: i32, resolution: i32) -> Axis {
        Axis {
            code,
            value,
            min,
            max,
            fuzz,
            flat,
            resolution
        }
    }

    pub const fn new_stick(code: AbsoluteAxisCode) -> Axis {
        Axis::new(code, 0, -32767, 32767, 16, 0, 0)
    }

    pub const fn new_trigger(code: AbsoluteAxisCode) -> Axis {
        Axis::new(code, 0, 0, 255, 0, 0, 0)
    }

    pub fn get(&self) -> UinputAbsSetup {
        UinputAbsSetup::new(self.code, self.get_abs_info())
    }

    fn get_abs_info(&self) -> AbsInfo {
        AbsInfo::new(self.value, self.min, self.max, self.fuzz, self.flat, self.resolution)
    }
}

const KEYS: &[KeyCode] = &[
    KeyCode::BTN_SOUTH,     // A
    KeyCode::BTN_RIGHT,     // B
    KeyCode::BTN_EAST,      // X
    KeyCode::BTN_NORTH,     // Y
    KeyCode::BTN_TL,        // Left Bumper
    KeyCode::BTN_TR,        // Right Bumper
    KeyCode::BTN_SELECT,    // Select
    KeyCode::BTN_START,     // Start
    KeyCode::BTN_THUMBL,    // Left Stick Button
    KeyCode::BTN_THUMBR,    // Right Stick Button
    KeyCode::BTN_DPAD_UP,   // D-Pad Up
    KeyCode::BTN_DPAD_DOWN, // D-Pad Down
    KeyCode::BTN_DPAD_LEFT, // D-Pad Left
    KeyCode::BTN_DPAD_RIGHT // D-Pad Right
];

const AXES: &[Axis] = &[
    Axis::new_stick(AbsoluteAxisCode::ABS_X),       // Left Stick X
    Axis::new_stick(AbsoluteAxisCode::ABS_Y),       // Left Stick Y
    Axis::new_stick(AbsoluteAxisCode::ABS_RX),      // Right Stick X
    Axis::new_stick(AbsoluteAxisCode::ABS_RY),      // Right Stick Y
    Axis::new_trigger(AbsoluteAxisCode::ABS_Z),     // Left Trigger
    Axis::new_trigger(AbsoluteAxisCode::ABS_RZ),    // Right Trigger
];

pub struct Device {
    device: VirtualDevice
}

impl Device {
    pub fn new(name: &str) -> Device {
        let mut attrset = AttributeSet::<KeyCode>::new();
        for key in KEYS {
            attrset.insert(*key);
        }

        let mut device = VirtualDevice::builder().unwrap()
            .name(name)
            .with_keys(&attrset).unwrap();

        for axis in AXES {
            device = device.with_absolute_axis(&axis.get()).unwrap();
        }

        let device = device.build().unwrap();

        Device {
            device
        }
    }

    pub fn set_controls(&mut self, controls: HashMap<String, f32>) {
        let mut events = Vec::new();
        for (key, value) in controls {
            if let Some(key) = self.string_to_key(&key) {
                events.push(InputEvent::new(EventType::KEY.0, key.0, value as i32));
            } else if let Some(axis) = self.string_to_axis(&key) {
                if key.contains("trigger") { events.push(InputEvent::new(EventType::ABSOLUTE.0, axis.0, (value * 255.0) as i32)); }
                else { events.push(InputEvent::new(EventType::ABSOLUTE.0, axis.0, (value * 32767.0) as i32)); }
            }
        }
        events.push(InputEvent::new(EventType::SYNCHRONIZATION.0, 0, 0));
        self.device.emit(&events).unwrap();
    }

    fn string_to_key(&self, key: &str) -> Option<KeyCode> {
        match key {
            "a" => Some(KeyCode::BTN_SOUTH),
            "b" => Some(KeyCode::BTN_RIGHT),
            "x" => Some(KeyCode::BTN_EAST),
            "y" => Some(KeyCode::BTN_NORTH),
            "l1" => Some(KeyCode::BTN_TL),
            "r1" => Some(KeyCode::BTN_TR),
            "minus" => Some(KeyCode::BTN_SELECT),
            "plus" => Some(KeyCode::BTN_START),
            "l3" => Some(KeyCode::BTN_THUMBL),
            "r3" => Some(KeyCode::BTN_THUMBR),
            "up" => Some(KeyCode::BTN_DPAD_UP),
            "down" => Some(KeyCode::BTN_DPAD_DOWN),
            "left" => Some(KeyCode::BTN_DPAD_LEFT),
            "right" => Some(KeyCode::BTN_DPAD_RIGHT),
            _ => None
        }
    }

    fn string_to_axis(&self, axis: &str) -> Option<AbsoluteAxisCode> {
        match axis {
            "left_y" => Some(AbsoluteAxisCode::ABS_Y),
            "left_x" => Some(AbsoluteAxisCode::ABS_X),
            "right_y" => Some(AbsoluteAxisCode::ABS_RY),
            "right_x" => Some(AbsoluteAxisCode::ABS_RX),
            "left_trigger" => Some(AbsoluteAxisCode::ABS_Z),
            "right_trigger" => Some(AbsoluteAxisCode::ABS_RZ),
            _ => None
        }
    }
}