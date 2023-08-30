use gtk::gdk::keys::constants;
use gtk::gdk::EventKey;
use gtk::glib::subclass::prelude::*;
use gtk::prelude::*;

pub fn set_keymaps(window: &super::SettingsWindow, key: &EventKey) -> gtk::Inhibit {
    match key.keyval() {
        constants::Tab => {
            if window.imp().notebook.current_page() == Some(2) {
                window.imp().notebook.set_page(0);
            } else {
                window.imp().notebook.next_page();
            }
            return Inhibit(true);
        }
        constants::ISO_Left_Tab => {
            if window.imp().notebook.current_page() == Some(0) {
                window.imp().notebook.set_page(2);
            } else {
                window.imp().notebook.prev_page();
            }
            return Inhibit(true);
        }
        constants::_1 => {
            window.imp().notebook.set_page(0);
        }
        constants::_2 => {
            window.imp().notebook.set_page(1);
        }
        constants::_3 => {
            window.imp().notebook.set_page(2);
        }
        _ => {}
    }
    Inhibit(false)
}
