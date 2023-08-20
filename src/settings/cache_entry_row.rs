mod imp;

use gtk::glib::subclass::prelude::*;
use gtk::glib::{self, clone, Object};
use gtk::prelude::*;

glib::wrapper! {
    pub struct CacheEntryRow(ObjectSubclass<imp::CacheEntryRow>)
    @extends gtk::Box, gtk::Widget,
    @implements gtk::Buildable, gtk::Orientable;
}

impl CacheEntryRow {
    pub fn new(name: String, content: String) -> Self {
        let object: Self = Object::builder().build();
        object.imp().show_entry_button.set_label(&name);
        object.imp().cache_content.set_text(&content);
        object.imp().revealer.set_reveal_child(false);
        object
            .imp()
            .show_entry_button
            .connect_clicked(clone!(@weak object => move |_| {
                let reveals = object.imp().revealer.reveals_child();
                object.imp().revealer.set_reveal_child(!reveals);
            }));
        object
    }
}
