use gio::prelude::ApplicationExtManual;
use gio::{ApplicationExt, FileOutputStream, OutputStreamWrite};
use glib::signal::Inhibit;
use gtk::prelude::{BuildableExtManual, ObjectExt};
use gtk::{ContainerExt, GtkWindowExt, TextViewExt, ToggleButtonExt, WidgetExt};
use lib::io::OpenOrCreate;
use std::cell::RefCell;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::rc::Rc;

static mut P: *mut BufWriter<File> = 0 as *const BufWriter<File> as *mut BufWriter<File>;

fn main() {
    let mut bw = BufWriter::new(File::open_or_create("./data").unwrap());
    unsafe {
        P = &bw as *const BufWriter<File> as *mut BufWriter<File>;
    }
    let app = gtk::Application::new(Some("pers.zhc.demo.a"), gio::ApplicationFlags::FLAGS_NONE)
        .expect("error");
    app.connect_activate(|app| {
        let window = gtk::ApplicationWindow::new(app);
        window.connect_event(|s, e| unsafe {
            let event_type = e.get_event_type();
            let bw = P.as_mut().unwrap();
            match event_type.to_string().as_str() {
                "EventType::MotionNotify" => {
                    let i = e.get_coords().unwrap();
                    s.set_title(format!("{:?}", i).as_str());
                    bw.write(format!("{} {}", i.0, i.1).as_bytes());
                    bw.write(&[b'\n']);
                }
                "EventType::Destroy" => {
                    bw.flush();
                }
                _ => {}
            }
            Inhibit::default()
        });
        window.show_all();
    });
    app.run(&[]);
}
