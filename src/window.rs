mod imp;

use glib::clone;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::gio::Cancellable;
use gtk::gio::MemoryInputStream;
use gtk::prelude::*;
use std::ops::Deref;

use glib::subclass::prelude::*;
use gtk::gio;
use gtk::Application;

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap,gtk::Buildable;
}

impl Window {
    pub fn new(app: &Application) -> Self {
        glib::Object::builder().property("application", app).build()
    }

    pub fn setup_buttons(&self) {
        let list_box = self.imp().list_box.deref();
        let button = gtk::Button::with_label("The silence of the lambs");
        list_box.add(&button);
        button.connect_clicked(clone!(@weak self as window => move |_| {
            window.imp().play_button.deref().show();
            let stream = MemoryInputStream::from_bytes(&glib::Bytes::from(include_bytes!("pictures/Loading_dark.png")));
            window.imp().poster.set_pixbuf(Some(&Pixbuf::from_stream(&stream, Cancellable::NONE).unwrap()));
        }));
    }
}
