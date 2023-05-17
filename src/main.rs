use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};

mod main_window;
mod generator;

fn main() {
    let application = Application::new(
        Some("com.example.gptgtk"),
        Default::default(),
    );

    application.connect_activate(|app| {
        main_window::build_ui(app);
    });

    application.run();
}
