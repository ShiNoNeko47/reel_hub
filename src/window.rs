mod imp;


use glib::Priority;
use glib::clone;
use glib::user_data_dir;
use gtk::Button;
use gtk::prelude::*;
use notify::{EventKind, Watcher, event::{CreateKind, ModifyKind, RemoveKind, RenameMode}};
use notify::recommended_watcher;
use std::ffi::OsStr;
use std::ops::Deref;
use std::path::Path;

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
        window.connect_key_press_event(|window, key| {
            match key.keycode() {
                Some(71) => {window.update(); gtk::Inhibit(true)}
                _ => gtk::Inhibit(false)
            }
        });

        window.update();
        let (sender, receiver) = glib::MainContext::channel(Priority::default());
        let mut watcher = recommended_watcher(move |event: Result<notify::Event, notify::Error>| {
            match event.as_ref().unwrap().kind {
                EventKind::Create(CreateKind::File) | EventKind::Remove(RemoveKind::File) => {
                    if event.as_ref().unwrap().paths.last().unwrap().extension() == Some(OsStr::new("mp4")) {
                        sender.send(()).unwrap();
                    }
                },
                EventKind::Modify(ModifyKind::Name(RenameMode::To)) => {
                    if event.as_ref().unwrap().paths.last().unwrap().is_dir() || event.as_ref().unwrap().paths.last().unwrap().extension() == Some(OsStr::new("mp4")){
                        sender.send(()).unwrap();
                    }
                },
                EventKind::Modify(ModifyKind::Name(RenameMode::From)) => {
                    sender.send(()).unwrap();
                },
                _ => {}
            };
        }).unwrap();

        receiver.attach(None, clone!(@weak window => @default-return Continue(false), move |_| {
            window.update();
            Continue(true)
        }));
        watcher.watch(Path::new(&movies::utils::user_dir(user_data_dir())), notify::RecursiveMode::Recursive).unwrap();
        window.imp().dir_watcher.replace(Some(watcher));
        window.imp().play_button.deref().connect_clicked(clone!(@weak window => move |_| {
            window.imp().movies.borrow()[window.imp().movie_selected.get().unwrap()].play(false);
        }));
        window
    }

    fn update(&self) {
        let mut movies = movies::detect::get_movies(movies::utils::user_dir(user_data_dir()));
        movies::utils::load_cache(&mut movies);
        self.imp().movies_len.replace(movies.len());
        self.imp().movies.replace(movies);
        self.setup_buttons();
    }

    fn setup_buttons(&self) {
        let list_box = self.imp().list_box.deref();
        list_box.forall(|widget| list_box.remove(widget));

        for movie in 0..self.imp().movies_len.get() {
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
        self.show_all();
    }
}
