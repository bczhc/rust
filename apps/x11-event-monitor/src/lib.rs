use chrono::Utc;
use serde::Serialize;

pub mod cli;
#[cfg(feature = "enabled")]
pub mod linux;

#[derive(Serialize)]
pub struct Event<'a> {
    time: i64,
    event: EventType<'a>,
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum EventType<'a> {
    MouseMotion {
        x: i32,
        y: i32,
    },
    KeyPress {
        scancode: u32,
        name: &'a str,
    },
    KeyRelease {
        scancode: u32,
        name: &'a str,
    },
    Button {
        name: &'a str,
        x: i32,
        y: i32,
    },
    MouseWheel {
        #[serde(rename = "xDelta")]
        x_delta: f32,
        #[serde(rename = "yDelta")]
        y_delta: f32,
    },
    Clipboard {
        text: &'a str,
        data: &'a [u8],
        #[serde(skip_serializing)]
        escaped: &'a str,
    },
    Selection {
        text: &'a str,
        data: &'a [u8],
        #[serde(skip_serializing)]
        escaped: &'a str,
    },
}

impl<'a> From<EventType<'a>> for Event<'a> {
    fn from(value: EventType<'a>) -> Self {
        Self {
            time: Utc::now().timestamp_millis(),
            event: value,
        }
    }
}

pub static mut JSON_FLAG: bool = false;

pub fn print_event(event: &Event) {
    if unsafe { JSON_FLAG } {
        println!("{}", serde_json::to_string(event).unwrap());
    } else {
        match event.event {
            EventType::MouseMotion { x, y } => {
                println!("MouseMotion {} {} {}", event.time, x, y);
            }
            EventType::KeyPress { scancode, name } => {
                println!("KeyPress {} {} {}", event.time, scancode, name);
            }
            EventType::KeyRelease { scancode, name } => {
                println!("KeyRelease {} {} {}", event.time, scancode, name);
            }
            EventType::Button { x, y, name } => {
                println!("Button {} {} {} {}", event.time, x, y, name);
            }
            EventType::MouseWheel { x_delta, y_delta } => {
                println!("MouseWheel {} {} {}", event.time, x_delta, y_delta);
            }
            EventType::Clipboard { escaped, .. } => {
                println!("Clipboard {} {}", event.time, escaped);
            }
            EventType::Selection { escaped, .. } => {
                println!("Selection {} {}", event.time, escaped);
            }
        }
    }
}
