use glib::subclass::prelude::*;
use gtk::gdk::EventKey;
use gtk::prelude::*;
use gtk::subclass::window::WindowImpl;

pub fn set_keymaps(window: &super::Window, key: &EventKey) -> gtk::Inhibit {
    match key.keycode() {
        Some(71) => {
            //<F5>
            window.update();
        }
        Some(36) => {
            // return
            window.imp().play_button.activate();
        }
        Some(38) => {
            // a
            window.imp().add_button.activate();
        }
        Some(56) => {
            // b
            window.imp().browse_button.activate();
        }
        Some(44) => {
            // j
            let button_selected = window.imp().button_selected.get();
            if button_selected < window.imp().buttons.borrow().len() - 1 {
                window.imp().button_selected.replace(button_selected + 1);
                window.set_focus(Some(&window.imp().buttons.borrow()[button_selected + 1]));
            }
        }
        Some(45) => {
            // k
            let button_selected = window.imp().button_selected.get();
            if button_selected > 0 {
                window.imp().button_selected.replace(button_selected - 1);
                window.set_focus(Some(&window.imp().buttons.borrow()[button_selected - 1]));
            }
        }
        Some(46) => {
            // l
            window.imp().activate_focus();
        }
        _ => {}
    }
    Inhibit(true)
}
