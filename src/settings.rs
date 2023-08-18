use gtk::subclass::prelude::*;
use gtk::traits::{ContainerExt, DialogExt, WidgetExt};
use gtk::{gio, Label};

use crate::main_window::Window;

mod imp;

gtk::glib::wrapper! {
    pub struct SettingsWindow(ObjectSubclass<imp::SettingsWindow>)
        @extends gtk::Box, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Buildable;
}

impl SettingsWindow {
    pub fn new(window: &Window) -> gtk::Dialog {
        let dialog: gtk::Dialog = gtk::Dialog::builder()
            .transient_for(window)
            .modal(true)
            .title("Settings")
            .build();
        let content: Self = gtk::glib::Object::builder().build();
        for plugin in window.imp().plugins.borrow().iter() {
            content
                .imp()
                .listbox_installed
                .add(&Label::new(Some(&plugin.1)));
        }
        dialog.content_area().add(&content);
        dialog.connect_delete_event(move |_, _| {
            content.close();
            gtk::Inhibit(false)
        });
        dialog
    }

    fn close(&self) {
        println!("Closing window");
    }
}
