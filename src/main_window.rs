mod imp;
mod keymaps;

use gtk::glib;
use gtk::glib::clone;
use gtk::glib::user_data_dir;
use gtk::glib::Priority;
use gtk::prelude::*;
use gtk::Button;
use gtk::CssProvider;
use gtk::DialogFlags;
use gtk::FileChooserAction;
use gtk::FileChooserDialog;
use gtk::MessageDialog;
use gtk::MessageType;
use gtk::ResponseType;
use notify::recommended_watcher;
use notify::{
    event::{CreateKind, ModifyKind, RemoveKind, RenameMode},
    EventKind, Watcher,
};
use reel_hub::detect;
use reel_hub::movie::ImageType;
use reel_hub::movie::Movie;
use reel_hub::movie::MovieData;
use reel_hub::utils;
use std::io::Write;
use std::ops::Deref;
use std::path::Path;
use std::process::ChildStdin;

use gtk::gio;
use gtk::glib::subclass::prelude::*;
use gtk::Application;

use reel_hub::res;

use crate::plugin;

gtk::glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap,gtk::Buildable;
}

impl Window {
    pub fn new(app: &Application) -> Self {
        let window: Self = gtk::glib::Object::builder()
            .property("application", app)
            .build();
        window.set_default_size(1000, 850);
        window.imp().poster.set_width_request(res::POSTER_W as i32);
        window.connect_key_press_event(keymaps::set_keymaps);

        window.connect_size_allocate(|window, _event| {
            window.autohide_backdrop();
        });

        let (sender, receiver) = gtk::glib::MainContext::channel(Priority::default());
        window.imp().plugins.replace(plugin::load_plugins(sender));
        receiver.attach(
            None,
            clone!(@weak window => @default-return Continue(false), move |response| {
                plugin::handle_response(response, &window);
                Continue(true)
            }),
        );

        window.update();
        window.setup_dir_watcher();

        window.imp().play_button.deref().set_label("  Play  ");
        window.imp().play_button.deref().connect_clicked(clone!(@weak window => move |button| {
            let movie = &window.imp().movies.borrow()[window.imp().movie_selected.get().unwrap()];
            let (sender, receiver) = glib::MainContext::channel(Priority::default());
            match movie.current_time {
                Some(0) | None => {
                    let mut handle = movie.play(false);
                    window.imp().status_label.deref().set_label(&format!("Playing: <b>{}</b>", movie.name));
                    std::thread::spawn(move || {
                        handle.wait().unwrap();
                        sender.send(()).unwrap();
                    });
                }
                Some(current_time) => {
                    let dialog = MessageDialog::new(
                        Some(&window),
                        DialogFlags::MODAL,
                        MessageType::Question,
                        gtk::ButtonsType::YesNo,
                        &format!("Continue watching from {}:{:02}:{:02}?", current_time / 3600, current_time / 60 % 60, current_time % 60));
                    dialog.set_decorated(false);
                    dialog.set_default_response(ResponseType::Yes);
                    dialog.show();
                    dialog.connect_response(clone!(@weak window => move |dialog, response| {
                        let movie = &window.imp().movies.borrow()[window.imp().movie_selected.get().unwrap()];
                        let mut handle;
                        match response {
                            ResponseType::Yes => {
                                handle = movie.play(true);
                                window.imp().status_label.deref().set_label(&format!("Playing: <b>{}</b>", movie.name));
                                window.imp().play_button.set_sensitive(false);
                                dialog.close();
                            }
                            ResponseType::No => {
                                handle = movie.play(false);
                                window.imp().status_label.deref().set_label(&format!("Playing: <b>{}</b>", movie.name));
                                window.imp().play_button.set_sensitive(false);
                                dialog.close();
                            }
                            _ => { return }
                        };
                        let sender = sender.clone();
                        std::thread::spawn(move || {
                            handle.wait().unwrap();
                            sender.send(()).unwrap();
                        });
                    }));
                }
            }

            receiver.attach(None, clone!(@weak button, @weak window => @default-return Continue(false), move |_| {
                let movie = window.imp().movie_selected.get().unwrap();
                let file_path = window.imp().movies.borrow()[movie].file.clone();
                let current_time = Movie::get_current_time(file_path);
                window.imp().movies.borrow_mut()[movie].current_time = current_time;
                if let None = current_time {
                    window.imp().movies.borrow_mut()[movie].done = true;
                } else {
                    window.imp().movies.borrow_mut()[movie].done = false;
                }
                window.update_progressbar(&window.imp().buttons.borrow()[movie], movie);
                window.imp().update_cache();
                button.set_sensitive(true);
                window.imp().status_label.deref().set_label("");
                window.sort_buttons(movie);
                window.setup_buttons();
                Continue(true)
            }));
        }));

        window
            .imp()
            .add_button
            .deref()
            .connect_clicked(clone!(@weak window => move |_| {
                window.add_dir();
            }));

        window.imp().browse_button.deref().connect_clicked(clone!(@weak window => move |_| {
            let filechooser = FileChooserDialog::new(Some("Browse"), Some(&window), gtk::FileChooserAction::CreateFolder);
            filechooser.set_current_folder(utils::user_dir(user_data_dir()));
            filechooser.set_decorated(false);
            filechooser.show();
        }));

        window
    }

    fn autohide_backdrop(&self) {
        if let Some(backdrop) = self.imp().backdrop.pixbuf() {
            if self.imp().backdrop_container.allocated_width() > backdrop.width() {
                self.imp().backdrop.show();
            } else {
                self.imp().backdrop.hide();
            }
        }
    }

    fn add_dir(&self) {
        let filechooser = FileChooserDialog::with_buttons(
            Some("Add movies"),
            Some(self),
            FileChooserAction::SelectFolder,
            &[("Add to library", ResponseType::Ok)],
        );
        filechooser.connect_response(clone!(@weak self as window => move |dialog, response| {
            match response {
                ResponseType::Ok => {
                    let filechooser = FileChooserDialog::with_buttons(
                        Some("Save Library Item"),
                        Some(&window),
                        FileChooserAction::Save,
                        &[("Save", ResponseType::Ok), ("Cancel", ResponseType::Cancel)]
                    );
                    filechooser.set_current_folder(utils::user_dir(user_data_dir()));
                    filechooser.connect_response(clone!(@weak dialog => move |savedialog, response| {
                        match response {
                            ResponseType::Ok => {
                                symlink::symlink_dir(dialog.file().unwrap().path().unwrap(), savedialog.file().unwrap().path().unwrap()).unwrap();
                                window.update();
                            },
                            _ => {}
                        }
                        savedialog.close();
                        dialog.close();
                    }));
                    filechooser.show();
                }
                _ => {}
            }
            dialog.hide();
        }));
        filechooser.show();
    }

    pub fn update(&self) {
        let mut movies = detect::get_movies(
            utils::user_dir(user_data_dir()),
            self.imp().movies.borrow_mut().to_vec(),
        );
        self.imp().plugins.replace(
            self.imp()
                .plugins
                .take()
                .into_iter()
                .filter_map(|mut plugin| {
                    if let Ok(_) = plugin.write_all(b"add\n") {
                        Some(plugin)
                    } else {
                        None
                    }
                })
                .collect::<Vec<ChildStdin>>(),
        );
        self.imp().cache.replace(utils::load_cache(&mut movies));
        movies.sort_unstable();
        match self.imp().movie_selected.get() {
            Some(movie_selected) => {
                let movie = movies
                    .iter()
                    .position(|x| &self.imp().movies.borrow()[movie_selected].id == &x.id);
                if movie.is_none() {
                    self.imp()
                        .movie_selected_tmp
                        .replace(Some(self.imp().movies.borrow()[movie_selected].id));
                }
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

        if self.imp().button_selected.get() >= self.imp().buttons.borrow().len()
            && self.imp().buttons.borrow().len() > 0
        {
            self.imp()
                .button_selected
                .replace(self.imp().buttons.borrow().len() - 1);
            self.imp().buttons.borrow().last().unwrap().grab_focus();
        }
    }

    pub fn setup_buttons(&self) {
        let list_box = self.imp().list_box.deref();
        list_box.forall(|widget| list_box.remove(widget));
        self.imp().buttons.borrow_mut().clear();

        for movie in 0..self.imp().movies_len.get() {
            let button = Button::builder()
                .label(self.imp().movies.borrow()[movie].name.clone())
                .focus_on_click(false)
                .build();
            list_box.add(&button);

            let (sender, receiver) = glib::MainContext::channel(Priority::default());
            button.connect_clicked(clone!(@weak self as window => move |_| {
                let (name, year) = (window.imp().movies.borrow()[movie].name.clone(), window.imp().movies.borrow()[movie].year);
                let sender = sender.clone();
                window.imp().movie_select(Some(movie));
                if window.imp().movies.borrow()[movie].data.is_none() {
                    std::thread::spawn(move || {
                        let data = MovieData::fetch_data(year, name);
                        sender.send((movie, data)).unwrap();
                    });
                }
                window.autohide_backdrop();
            }));
            if movie == self.imp().button_selected.get() {
                button.grab_focus();
            }
            self.update_progressbar(&button, movie);
            self.imp().buttons.borrow_mut().push(button);
            receiver.attach(None, clone!(@weak self as window => @default-return Continue(false), move |(movie, data)| {
                match data {
                    Some(data) => {
                        window.imp().movies.borrow_mut()[movie].data.replace(data);
                        window.imp().movie_select(Some(movie));
                        window.update_progressbar(&window.imp().buttons.borrow()[movie], movie);
                    }
                    None => {
                        window.imp().poster.deref().set_pixbuf(Some(&res::check_connection(&ImageType::Poster)));
                        window.imp().backdrop.deref().set_pixbuf(Some(&res::check_connection(&ImageType::Backdrop)));
                    }
                }
                window.autohide_backdrop();
                Continue(true)
            }));
        }
        self.show_all();
        self.autohide_backdrop();
    }

    fn sort_buttons(&self, movie: usize) {
        let movie_current = &self.imp().movies.borrow()[movie].clone();
        self.imp().movies.borrow_mut().sort_unstable();
        let idx = self
            .imp()
            .movies
            .borrow()
            .iter()
            .position(|x| movie_current == x);
        self.imp().movie_select(idx);
        self.imp().button_selected.replace(idx.unwrap_or(0));
    }

    fn update_progressbar(&self, button: &Button, movie: usize) {
        if self.imp().movies.borrow()[movie].done {
            let css_provider = CssProvider::new();
            css_provider
                .load_from_data(
                    format!(
                        "button {{
                            background: #0f0f0f,
                                linear-gradient(to right, @accent, @accent) 5px calc(100% - 5px) / calc(100% - 10px) 1px no-repeat;
                        }}",
                    )
                    .as_bytes(),
                )
                .unwrap();
            button
                .style_context()
                .add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
        } else if let Some(progress) = self.imp().movies.borrow()[movie].get_progress() {
            let css_provider = CssProvider::new();
            css_provider
                .load_from_data(
                    format!(
                        "button {{
                            background: #0f0f0f,
                                linear-gradient(to right, @accent {progress}%, black {progress}%) 5px calc(100% - 5px) / calc(100% - 10px) 1px no-repeat;
                        }}",
                    )
                    .as_bytes(),
                )
                .unwrap();
            button
                .style_context()
                .add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
        }
    }

    fn setup_dir_watcher(&self) {
        let (sender, receiver) = glib::MainContext::channel(Priority::default());
        let mut watcher =
            recommended_watcher(move |event: Result<notify::Event, notify::Error>| {
                match event.as_ref().unwrap().kind {
                    EventKind::Create(CreateKind::File) | EventKind::Remove(RemoveKind::File) => {
                        if res::check_filetype(
                            event.as_ref().unwrap().paths.last().unwrap().extension(),
                        ) {
                            sender.send(()).unwrap();
                        }
                    }
                    EventKind::Modify(ModifyKind::Name(RenameMode::To)) => {
                        if event.as_ref().unwrap().paths.last().unwrap().is_dir()
                            || res::check_filetype(
                                event.as_ref().unwrap().paths.last().unwrap().extension(),
                            )
                        {
                            sender.send(()).unwrap();
                        }
                    }
                    EventKind::Modify(ModifyKind::Name(RenameMode::From)) => {
                        sender.send(()).unwrap();
                    }
                    _ => {}
                };
            })
            .unwrap();

        receiver.attach(
            None,
            clone!(@weak self as window => @default-return Continue(false), move |_| {
                window.update();
                Continue(true)
            }),
        );
        watcher
            .watch(
                Path::new(&utils::user_dir(user_data_dir())),
                notify::RecursiveMode::Recursive,
            )
            .unwrap();
        self.imp().dir_watcher.replace(Some(watcher));
    }
}
