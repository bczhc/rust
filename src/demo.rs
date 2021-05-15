use gio::prelude::ApplicationExtManual;
use gio::ApplicationExt;
use glib::signal::Inhibit;
use gtk::prelude::{BuildableExtManual, ObjectExt};
use gtk::{ContainerExt, GtkWindowExt, TextViewExt, ToggleButtonExt, WidgetExt};

fn main() {
    let app = gtk::Application::new(Some("pers.zhc.demo.a"), gio::ApplicationFlags::FLAGS_NONE)
        .expect("error");
    app.connect_activate(|app| {
        let window = gtk::ApplicationWindow::new(app);
        let tv = gtk::TextView::new();
        tv.set_editable(true);
        window.add(&tv);
        window.set_title("hello");
        tv.connect_event(|s, e| {
            println!("{:?}", e);
            Inhibit::default()
        });
        window.connect_event(|s, e| {
            if e.get_event_type().to_string() == "EventType::MotionNotify" {
                let i = e.get_coords().unwrap();
                s.set_title(format!("{:?}", i).as_str());
            }
            Inhibit::default()
        });
        window.show_all();
    });
    app.run(&[]);
}
