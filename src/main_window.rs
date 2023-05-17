use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, HeaderBar, Entry, ComboBoxText, TextView, ScrolledWindow, Button, TextBuffer};
use reqwest::Client;
use std::error::Error;
use serde_json::Value;
use glib::prelude::*;
use gtk::gdk::key;
use gtk::gdk::keys::constants::Return;
use gdk::EventKey;
use gtk::atk::RelationType::Null;
use gtk::Inhibit;
use std::sync::Arc;
use std::sync::Mutex;
use gtk::TextTagTable;

pub fn build_ui(application: &Application) {
    let window = ApplicationWindow::new(application);
    window.set_title("GPT GTK");
    window.set_default_size(640, 480);
    window.set_decorated(false);

    let headerbar = HeaderBar::new();
    headerbar.set_show_close_button(true);
    headerbar.set_title(Some("GPT GTK"));

    let model_combo = ComboBoxText::new();

    let endpoint_buffer = gtk::EntryBuffer::new(None);
    let endpoint_entry = Entry::with_buffer(&endpoint_buffer);
    endpoint_entry.set_placeholder_text(Some("API endpoint"));

    let model_combo_arc = Arc::new(Mutex::new(model_combo.clone()));
    let endpoint_buffer_arc = Arc::new(endpoint_buffer);
    endpoint_entry.connect_activate(move |entry| {
        let endpoint = endpoint_buffer_arc.text();
        let model_combo = model_combo_arc.lock().unwrap();

        if let Ok(models) = fetch_models(&endpoint) {
            model_combo.remove_all();
            for model in models {
                model_combo.append_text(&model);
            }
        } else {
            eprintln!("Failed to fetch models from {}", endpoint);
        }
    });
    
    headerbar.pack_start(&endpoint_entry);
    headerbar.pack_end(&model_combo);

    let scrolled_window = ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    scrolled_window.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);

    let text_view = TextView::new();
    let text_view_buffer = TextBuffer::new(Some(&TextTagTable::new()));
    text_view.set_editable(false);
    text_view.set_buffer(Some(&text_view_buffer));
    text_view.set_cursor_visible(false);

    scrolled_window.add(&text_view);

    let send_button = Button::with_label("Send");

    let input_entry = Entry::new();
    input_entry.set_placeholder_text(Some("Input text"));

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 5);
    vbox.pack_start(&scrolled_window, true, true, 0);
    vbox.pack_start(&input_entry, false, false, 0);
    vbox.pack_start(&send_button, false, false, 0);

    let bigvbox = gtk::Box::new(gtk::Orientation::Vertical, 5);
    bigvbox.pack_start(&headerbar, false, false, 0);

    bigvbox.pack_start(&vbox, true, true, 0);

    window.add(&bigvbox);
    window.show_all();
}

fn fetch_models(endpoint: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let models_url = format!("{}/v1/models", endpoint);
    let response = reqwest::blocking::get(&models_url)?;
    let json: Value = serde_json::from_str(&response.text()?)?;
    let models = json["data"]
        .as_array()
        .ok_or("Failed to parse models list")?
        .iter()
        .map(|model| model["id"].as_str().unwrap().to_string())
        .collect();
    Ok(models)
}
