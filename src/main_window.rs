use gdk::cairo::FontOptions;
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
use crate::generator::get_chat_response;

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
    endpoint_entry.set_placeholder_text(Some("Endpoint address"));

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
    text_view.set_wrap_mode(gtk::WrapMode::Word);
    let text_view_buffer = TextBuffer::new(Some(&TextTagTable::new()));
    text_view.set_editable(false);
    text_view.set_buffer(Some(&text_view_buffer));
    text_view.set_cursor_visible(false);

    scrolled_window.add(&text_view);

    let send_button = Button::with_label("Send");

    let input_buffer = gtk::EntryBuffer::new(None);
    let input_entry = Entry::with_buffer(&input_buffer);
    input_entry.set_placeholder_text(Some("Write a message here..."));
    // hell hack
    let input_entry_arc = Arc::new(input_entry.clone());
    let input_buffer_arc = Arc::new(input_buffer);
    input_entry.connect_activate(move |entry| {
        let model = model_combo.active_text().unwrap().to_string();
        let temp = "0.7";
        let init = "I am a chatbot.";
        let prompt = input_buffer_arc.text();

        if let Ok(response) = get_chat_response(&model, &temp, &init, &prompt) {
            // Update the text view with the response
            text_view_buffer.insert_at_cursor(&response);
        } else {
            eprintln!("Failed to get chat response");
        }
    });


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
