use std::thread::spawn;

use winit::event::{DeviceEvent, ElementState, MouseScrollDelta};
use winit::event_loop::ControlFlow;
use x11_clipboard::Clipboard;

use x11_event_monitor::cli::build_cli;
use x11_event_monitor::{print_event, EventType, XDo, JSON_FLAG};

const KEYS: &str = include_str!("./keys");

fn main() {
    let matches = build_cli().get_matches();
    let json_flag = matches.get_flag("json");
    unsafe {
        JSON_FLAG = json_flag;
    }

    spawn(start_clipboard_monitor);
    spawn(start_primary_selection_monitor);

    let mut keys = vec![None; 256];
    for line in KEYS.lines().filter(|x| !x.is_empty()) {
        let mut split = line.split_whitespace();
        let name = split.next().unwrap();
        let code = split.next().unwrap().parse::<usize>().unwrap();
        keys[code] = Some(String::from(name));
    }

    let xdo = XDo::new().unwrap();

    let event_loop = winit::event_loop::EventLoop::new();
    event_loop.run(move |e, _, cf| {
        *cf = ControlFlow::Wait;

        if let winit::event::Event::DeviceEvent { event, .. } = e {
            match event {
                DeviceEvent::MouseMotion { .. } => {
                    let location = xdo.mouse_location().unwrap();
                    print_event(
                        &EventType::MouseMotion {
                            x: location.0,
                            y: location.1,
                        }
                        .into(),
                    );
                }
                DeviceEvent::MouseWheel {
                    delta: MouseScrollDelta::LineDelta(x, y),
                } => {
                    print_event(
                        &EventType::MouseWheel {
                            x_delta: x,
                            y_delta: y,
                        }
                        .into(),
                    );
                }
                DeviceEvent::Button { button, state } => {
                    if state == ElementState::Pressed {
                        let location = xdo.mouse_location().unwrap();
                        let button_name = match button {
                            1 => "Left",
                            2 => "Middle",
                            3 => "Right",
                            _ => "Unknown",
                        };
                        print_event(
                            &EventType::Button {
                                name: button_name,
                                x: location.0,
                                y: location.1,
                            }
                            .into(),
                        );
                    }
                }
                DeviceEvent::Key(k) => {
                    if k.state == ElementState::Pressed || k.state == ElementState::Released {
                        let code_name = keys[k.scancode as usize].as_ref();
                        let code_name = match code_name {
                            None => "UNKNOWN",
                            Some(s) => s,
                        };
                        match k.state {
                            ElementState::Pressed => {
                                print_event(
                                    &EventType::KeyPress {
                                        name: code_name,
                                        scancode: k.scancode,
                                    }
                                    .into(),
                                );
                            }
                            ElementState::Released => print_event(
                                &EventType::KeyRelease {
                                    name: code_name,
                                    scancode: k.scancode,
                                }
                                .into(),
                            ),
                        };
                    }
                }
                _ => {}
            }
        }
    });
}

fn start_clipboard_monitor() {
    let clipboard = Clipboard::new().unwrap();

    loop {
        let Ok(val) = clipboard
            .load_wait(
                clipboard.setter.atoms.clipboard,
                clipboard.setter.atoms.string,
                clipboard.setter.atoms.property,
            ) else {
            continue
        };

        let escaped = bczhc_lib::str::escape_utf8_bytes(&val);

        let text = String::from_utf8_lossy(&val).to_string();

        print_event(
            &EventType::Clipboard {
                text: &text,
                data: &val,
                escaped: &escaped,
            }
            .into(),
        );
    }
}

fn start_primary_selection_monitor() {
    let clipboard = Clipboard::new().unwrap();
    loop {
        let Ok(vec) = clipboard.load_wait(
            clipboard.getter.atoms.primary,
            clipboard.getter.atoms.utf8_string,
            clipboard.getter.atoms.property,
        ) else {
            continue
        };

        let escaped = bczhc_lib::str::escape_utf8_bytes(&vec);

        let text = String::from_utf8_lossy(&vec).to_string();
        print_event(
            &EventType::Selection {
                text: &text,
                data: &vec,
                escaped: &escaped,
            }
            .into(),
        );
    }
}
