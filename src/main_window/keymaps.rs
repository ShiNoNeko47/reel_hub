use gtk::gdk::keys::constants;
use gtk::gdk::EventKey;
use gtk::glib::subclass::prelude::*;
use gtk::prelude::*;
use gtk::subclass::window::WindowImpl;

use crate::settings::SettingsWindow;

pub fn set_keymaps(window: &super::Window, key: &EventKey) -> gtk::Inhibit {
    if window.imp().revealer_search.reveals_child() {
        match key.keyval() {
            constants::Escape => {
                window.imp().revealer_search.set_reveal_child(false);
                window.imp().entry_search.buffer().set_text("");
            }
            constants::Return => {
                window.imp().revealer_search.set_reveal_child(false);
            }
            _ => {
                return Inhibit(false);
            }
        }
        let first = window
            .imp()
            .movies
            .borrow()
            .iter()
            .position(|movie| window.filter(movie.name.clone()));
        if let Some(first) = first {
            window.imp().buttons.borrow()[first].grab_focus();
            window.imp().button_selected.replace(first);
        }
        return Inhibit(true);
    }
    match key.keyval() {
        constants::F5 | constants::r => {
            window.update();
        }
        constants::Return => {
            if window.imp().play_button.is_visible() {
                window.imp().play_button.activate();
            }
        }
        constants::a => {
            window.imp().add_button.activate();
        }
        constants::b => {
            window.imp().browse_button.activate();
        }
        constants::h | constants::Left => {
            window.imp().revealer.set_reveal_child(false);
        }
        constants::j | constants::Down => {
            let button_selected = window.imp().button_selected.get();
            let buttons = window.imp().buttons.borrow();
            let mut buttons = buttons
                .iter()
                .enumerate()
                .filter(|(_, button)| window.filter(button.label().unwrap().to_string()));
            buttons.find(|(i, _)| i == &button_selected);
            if let Some(button) = buttons.next() {
                window.imp().button_selected.replace(button.0);
                button.1.grab_focus();
            }
        }
        constants::k | constants::Up => {
            let button_selected = window.imp().button_selected.get();
            let buttons = window.imp().buttons.borrow();
            let mut buttons = buttons
                .iter()
                .enumerate()
                .filter(|(_, button)| window.filter(button.label().unwrap().to_string()))
                .rev();
            buttons.find(|(i, _)| i == &button_selected);
            if let Some(button) = buttons.next() {
                window.imp().button_selected.replace(button.0);
                button.1.grab_focus();
            }
        }
        constants::l | constants::Right => {
            if window.imp().revealer.reveals_child() {
                window.imp().activate_focus();
            } else {
                window.imp().revealer.set_reveal_child(true);
                window.imp().buttons.borrow()[window.imp().button_selected.get()].grab_focus();
            }
        }
        constants::semicolon => {
            window.imp().movies.borrow_mut()[window.imp().button_selected.get()].done = false;
            window.setup_buttons();
        }
        constants::g => {
            let buttons = window.imp().buttons.borrow();
            let first = buttons
                .iter()
                .enumerate()
                .filter(|(_, button)| window.filter(button.label().unwrap().to_string().clone()))
                .next();
            if let Some(first) = first {
                window.imp().button_selected.replace(first.0);
                first.1.grab_focus();
            }
        }
        constants::G => {
            let buttons = window.imp().buttons.borrow();
            let last = buttons
                .iter()
                .enumerate()
                .filter(|(_, button)| window.filter(button.label().unwrap().to_string().clone()))
                .last();
            if let Some(last) = last {
                window.imp().button_selected.replace(last.0);
                last.1.grab_focus();
            }
        }
        constants::Escape => {
            let settings_window = SettingsWindow::new(window);
            settings_window.show_all();
        }
        constants::slash => {
            window.imp().revealer_search.set_reveal_child(true);
            window.imp().entry_search.grab_focus();
        }
        _ => {}
    }
    if let Some(key) = key.keyval().to_unicode() {
        window.plugin_broadcast(format!("key;{key}"));
    }
    Inhibit(true)
}
