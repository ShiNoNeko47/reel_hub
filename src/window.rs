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
        window.setup_dir_watcher();

        window.imp().play_button.deref().set_label("  Play  ");
        window.imp().play_button.deref().connect_clicked(clone!(@weak window => move |_| {
            window.imp().movies.borrow()[window.imp().movie_selected.get().unwrap()].play(false);
        }));
        window
    }

    fn update(&self) {
        let mut movies = movies::detect::get_movies(movies::utils::user_dir(user_data_dir()));
        movies::utils::load_cache(&mut movies);
        match self.imp().movie_selected.get() {
            Some(movie_selected) => {
                let movie = movies.iter().position(|x| &self.imp().movies.borrow()[movie_selected] == x);
                self.imp().movies_len.replace(movies.len());
                self.imp().movies.replace(movies);
                self.imp().movie_select(movie);
            }
            None => {
                self.imp().movies_len.replace(movies.len());
                self.imp().movies.replace(movies);
            }
        }

        self.setup_buttons();
    }

    fn setup_buttons(&self) {
        let list_box = self.imp().list_box.deref();
        list_box.forall(|widget| list_box.remove(widget));

        for movie in 0..self.imp().movies_len.get() {
            let button = Button::builder()
                .label(self.imp().movies.borrow()[movie].name.clone()).build();
            list_box.add(&button);

            let (sender, receiver) = glib::MainContext::channel(Priority::default());
            button.connect_clicked(clone!(@weak self as window => move |_| {
                let (name, year) = (window.imp().movies.borrow()[movie].name.clone(), window.imp().movies.borrow()[movie].year);
                let sender = sender.clone();
                window.imp().movie_select(Some(movie));
                if window.imp().movies.borrow()[movie].data.is_none() {
                    std::thread::spawn(move || {
                        let data = movies::movie::MovieData::fetch_data(year, name);
                        sender.send((movie, data)).unwrap();
                    });
                }
            }));
            receiver.attach(None, clone!(@weak self as window => @default-return Continue(false), move |(movie, data)| {
                window.imp().movies.borrow_mut()[movie].data.replace(data.unwrap());
                window.imp().movie_select(Some(movie));
                Continue(true)
            }));
        }
        self.show_all();
    }
    
    fn setup_dir_watcher(&self) {
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

        receiver.attach(None, clone!(@weak self as window => @default-return Continue(false), move |_| {
            window.update();
            Continue(true)
        }));
        watcher.watch(Path::new(&movies::utils::user_dir(user_data_dir())), notify::RecursiveMode::Recursive).unwrap();
        self.imp().dir_watcher.replace(Some(watcher));
    }
}
