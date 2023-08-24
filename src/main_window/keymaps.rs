use gtk::gdk::keys::constants;
use gtk::gdk::EventKey;
use gtk::glib::subclass::prelude::*;
use gtk::prelude::*;
use gtk::subclass::window::WindowImpl;

use crate::settings::SettingsWindow;

pub fn set_keymaps(window: &super::Window, key: &EventKey) -> gtk::Inhibit {
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
            if button_selected < window.imp().buttons.borrow().len() - 1 {
                window.imp().button_selected.replace(button_selected + 1);
                window.imp().buttons.borrow()[button_selected + 1].grab_focus();
            }
        }
        constants::k | constants::Up => {
            let button_selected = window.imp().button_selected.get();
            if button_selected > 0 {
                window.imp().button_selected.replace(button_selected - 1);
                window.imp().buttons.borrow()[button_selected - 1].grab_focus();
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
            window.imp().button_selected.replace(0);
            window.imp().buttons.borrow()[window.imp().button_selected.get()].grab_focus();
        }
        constants::G => {
            window
                .imp()
                .button_selected
                .replace(window.imp().buttons.borrow().len() - 1);
            window.imp().buttons.borrow()[window.imp().button_selected.get()].grab_focus();
        }
        constants::Escape => {
            let settings_window = SettingsWindow::new(window);
            settings_window.show_all();
        }
        _ => {}
    }
    if let Some(key) = key.keyval().to_unicode() {
        window.plugin_broadcast(format!("key;{key}"));
    }
    Inhibit(true)
}
