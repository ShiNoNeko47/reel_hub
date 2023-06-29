mod imp;

use glib::clone;
use glib::user_data_dir;
use gtk::Button;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::gio::Cancellable;
use gtk::gio::MemoryInputStream;
use gtk::prelude::*;
use std::fs;
use std::ops::Deref;
use std::path::PathBuf;

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
        let movies = movies::detect::get_movies(user_dir(user_data_dir()));
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
            self.imp().movies.borrow_mut()[movie].fetch_data();
            let button = Button::builder()
                .label(self.imp().movies.borrow()[movie].name.clone()).build();
            list_box.add(&button);

            button.connect_clicked(clone!(@weak self as window => move |_| {
                let data = window.imp().movies.borrow()[movie].data.clone();

                window.imp().original_title.deref().set_label(&format!("<b>Title:</b> {}", data.as_ref().unwrap().original_title));
                window.imp().original_language.deref().set_label(&format!("<b>Original Language:</b> {}", data.as_ref().unwrap().original_language));
                window.imp().overview.deref().set_label(&format!("<b>Overview:</b> {}", data.as_ref().unwrap().overview));
                window.imp().vote_average.deref().set_label(&format!("<b>Vote Average:</b> {}", data.as_ref().unwrap().vote_average.to_string()));
                window.imp().vote_count.deref().set_label(&format!("<b>Vote Count:</b> {}", data.as_ref().unwrap().vote_count.to_string()));
                window.imp().release_date.deref().set_label(&format!("<b>Release Date:</b> {}", data.as_ref().unwrap().release_date));

                window.imp().movie_selected.replace(Some(movie));
                window.imp().play_button.deref().show();
            }));
        }

        // let button = gtk::Button::with_label("The silence of the lambs");
        // button.connect_clicked(clone!(@weak self as window => move |_| {
        //     window.imp().play_button.deref().show();
        //     window.imp().poster.set_pixbuf(Some(&loading_pixbuf()));
        // }));
    }
}

fn user_dir(path: PathBuf) -> String {
    let mut path: PathBuf = path;
    path.push("movies");
    fs::create_dir_all(&path).expect("Couldn't create directory");
    path.to_str().unwrap().to_string()
}

fn loading_pixbuf() -> Pixbuf {
    let stream = MemoryInputStream::from_bytes(&glib::Bytes::from(include_bytes!("pictures/Loading_dark.png")));
    Pixbuf::from_stream(&stream, Cancellable::NONE).unwrap()
}
