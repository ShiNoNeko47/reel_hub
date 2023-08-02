use glib::subclass::prelude::*;
use gtk::gdk::EventKey;
use gtk::prelude::*;
use gtk::subclass::window::WindowImpl;
use std::io::Write;

pub fn set_keymaps(window: &super::Window, key: &EventKey) -> gtk::Inhibit {
    match key.keycode() {
        Some(71) | Some(27) => {
            // <F5> or r
            window.update();
        }
        Some(36) => {
            // return
            if window.imp().play_button.is_visible() {
                window.imp().play_button.activate();
            }
        }
        Some(38) => {
            // a
            window.imp().add_button.activate();
        }
        Some(56) => {
            // b
            window.imp().browse_button.activate();
        }
        Some(43) | Some(113) => {
            // h or left
            window.imp().revealer.set_reveal_child(false);
        }
        Some(44) | Some(116) => {
            // j or down
            let button_selected = window.imp().button_selected.get();
            if button_selected < window.imp().buttons.borrow().len() - 1 {
                window.imp().button_selected.replace(button_selected + 1);
                window.imp().buttons.borrow()[button_selected + 1].grab_focus();
            }
        }
        Some(45) | Some(111) => {
            // k or up
            let button_selected = window.imp().button_selected.get();
            if button_selected > 0 {
                window.imp().button_selected.replace(button_selected - 1);
                window.imp().buttons.borrow()[button_selected - 1].grab_focus();
            }
        }
        Some(46) | Some(114) => {
            // l or right
            if window.imp().revealer.reveals_child() {
                window.imp().activate_focus();
            } else {
                window.imp().revealer.set_reveal_child(true);
                window.imp().buttons.borrow()[window.imp().button_selected.get()].grab_focus();
            }
        }
        Some(47) => {
            // ; (semicolon)
            window.imp().movies.borrow_mut()[window.imp().button_selected.get()].done = false;
            window.setup_buttons();
        }
        Some(42) => {
            // g
            if key.keyval().is_upper() {
                window
                    .imp()
                    .button_selected
                    .replace(window.imp().buttons.borrow().len() - 1);
            } else {
                window.imp().button_selected.replace(0);
            }
            window.imp().buttons.borrow()[window.imp().button_selected.get()].grab_focus();
        }
        Some(41) => {
            for plugin in window.imp().plugins.borrow_mut().iter_mut() {
                if let Err(error) = plugin.write_all(b"asdf\n") {
                    eprintln!("Error writing to plugin: {:?}", error);
                }
            }
        }
        _ => {}
    }
    Inhibit(true)
}
