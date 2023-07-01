mod imp;

use glib::clone;
use glib::user_data_dir;
use gtk::Button;
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
        let window: Self = glib::Object::builder().property("application", app).build();
        let movies = movies::detect::get_movies(movies::movie::user_dir(user_data_dir()));
        window.imp().movies_len.replace(movies.len());
        window.imp().movies.replace(movies);
        window.imp().play_button.deref().connect_clicked(clone!(@weak window => move |_| {
            window.imp().movies.borrow()[window.imp().movie_selected.get().unwrap()].play(false);
        }));
        window
    }

    pub fn setup_buttons(&self) {
        let list_box = self.imp().list_box.deref();

        for movie in 0..self.imp().movies_len.get() {
            // self.imp().movies.borrow_mut()[movie].fetch_data();
            let button = Button::builder()
                .label(self.imp().movies.borrow()[movie].name.clone()).build();
            list_box.add(&button);

            button.connect_clicked(clone!(@weak self as window => move |_| {
                if window.imp().movies.borrow()[movie].data.is_none() {
                    window.imp().movies.borrow_mut()[movie].fetch_data();
                }
                window.imp().movie_select(movie);
            }));
        }
    }
}
