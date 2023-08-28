mod imp;
mod keymaps;

use crate::detect;
use crate::movie::ImageType;
use crate::movie::Movie;
use crate::movie::MovieData;
use crate::utils;
use gtk::glib;
use gtk::glib::clone;
use gtk::glib::user_data_dir;
use gtk::glib::Priority;
use gtk::glib::Sender;
use gtk::prelude::*;
use gtk::Button;
use gtk::CssProvider;
use gtk::Dialog;
use gtk::DialogFlags;
use gtk::Entry;
use gtk::FileChooserAction;
use gtk::FileChooserDialog;
use gtk::Label;
use gtk::ListBox;
use gtk::MessageDialog;
use gtk::MessageType;
use gtk::ResponseType;
use notify::recommended_watcher;
use notify::{
    event::{CreateKind, ModifyKind, RemoveKind, RenameMode},
    EventKind, Watcher,
};
use std::io::Write;
use std::ops::Deref;
use std::path::Path;
use std::process::ChildStdin;

use gtk::gio;
use gtk::glib::subclass::prelude::*;
use gtk::Application;

use crate::res;

use crate::plugin;

pub enum UserInputType {
    Text,
    Password,
    Choice,
}

gtk::glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Buildable;
}

impl Window {
    pub fn new(app: &Application) -> Self {
        let window: Self = gtk::glib::Object::builder()
            .property("application", app)
            .build();
        window.set_default_size(1000, 850);
        window
            .imp()
            .poster
            .set_width_request(window.imp().settings.borrow().poster_w as i32);
        window.connect_key_press_event(keymaps::set_keymaps);

        window.connect_size_allocate(|window, _event| {
            window.autohide_backdrop();
        });

        let (sender, receiver) = gtk::glib::MainContext::channel(Priority::default());
        window.imp().plugins.replace(plugin::load_plugins(sender));
        receiver.attach(
            None,
            clone!(@weak window => @default-return Continue(false), move |(response, plugin_id)| {
                plugin::handle_response(response, &window, plugin_id);
                Continue(true)
            }),
        );

        window.update();
        window.setup_dir_watcher();

        window.imp().play_button.deref().set_label("  Play  ");
        window.imp().play_button.deref().connect_clicked(clone!(@weak window => move |button| {
            let idx = window.imp().movie_selected.get().unwrap();
            let movie = &window.imp().movies.borrow()[idx];
            let (sender, receiver) = glib::MainContext::channel(Priority::default());
            match movie.current_time {
                Some(0) | None => {
                    let mut handle = movie.play(false, &window.imp().settings.borrow().player_args);
                    window.imp().status_label.deref().set_label(&format!("Playing: <b>{}</b>", movie.name));
                    window.plugin_broadcast(format!("playing;{}", movie.id));
                    std::thread::spawn(move || {
                        handle.wait().unwrap();
                        sender.send(idx).unwrap();
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
                        let movie = &window.imp().movies.borrow()[idx];
                        let mut handle;
                        match response {
                            ResponseType::Yes => {
                                handle = movie.play(true, &window.imp().settings.borrow().player_args);
                                window.imp().status_label.deref().set_label(&format!("Playing: <b>{}</b>", movie.name));
                                window.imp().play_button.set_sensitive(false);
                                dialog.close();
                            }
                            ResponseType::No => {
                                handle = movie.play(false, &window.imp().settings.borrow().player_args);
                                window.imp().status_label.deref().set_label(&format!("Playing: <b>{}</b>", movie.name));
                                window.imp().play_button.set_sensitive(false);
                                dialog.close();
                            }
                            _ => { return }
                        };
                        let sender = sender.clone();
                        window.plugin_broadcast(format!("playing;{}", movie.id));
                        std::thread::spawn(move || {
                            handle.wait().unwrap();
                            sender.send(idx).unwrap();
                        });
                    }));
                }
            }

            receiver.attach(None, clone!(@weak button, @weak window => @default-return Continue(false), move |idx| {
                let file_path = window.imp().movies.borrow()[idx].file.clone();
                let current_time = Movie::get_current_time(file_path);
                window.imp().movies.borrow_mut()[idx].current_time = current_time;
                if let Some(current_time) = current_time {
                    window.imp().movies.borrow_mut()[idx].done = false;
                    window.plugin_broadcast(format!("quit;{};{}", current_time, window.imp().movies.borrow()[idx].id));
                } else {
                    window.imp().movies.borrow_mut()[idx].done = true;
                    window.plugin_broadcast(format!("quit;;{}", window.imp().movies.borrow()[idx].id));
                }
                window.update_progressbar(&window.imp().buttons.borrow()[idx], idx);
                window.imp().update_cache();
                button.set_sensitive(true);
                window.imp().status_label.deref().set_label("");
                window.sort_buttons(idx);
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

    pub fn plugin_broadcast(&self, request: String) {
        self.imp().plugins.replace(
            self.imp()
                .plugins
                .take()
                .into_iter()
                .map(|mut plugin| {
                    if let Err(_) = plugin.0.write_all(format!("{request}\n").as_bytes()) {
                        plugin.2 = false;
                    }
                    plugin
                })
                .collect::<Vec<(ChildStdin, String, bool)>>(),
        );
    }

    pub fn update(&self) {
        let mut movies = detect::get_movies(
            utils::user_dir(user_data_dir()),
            self.imp().movies.borrow_mut().to_vec(),
        );
        self.plugin_broadcast("add".to_string());
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
                if let Some(movie) = movie {
                    self.plugin_broadcast(format!(
                        "selected;{}",
                        self.imp().movies.borrow()[movie].id
                    ));
                } else {
                    self.plugin_broadcast("selected;".to_string());
                }
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
                window.plugin_broadcast(format!(
                    "selected;{}",
                    window.imp().movies.borrow()[movie].id
                ));
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
                window.plugin_broadcast(format!(
                    "selected;{}",
                    window.imp().movies.borrow()[movie].id
                ));
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
        if let Some(idx) = idx {
            self.plugin_broadcast(format!("selected;{}", self.imp().movies.borrow()[idx].id));
        } else {
            self.plugin_broadcast("selected;".to_string());
        }
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

    pub fn get_user_input(
        &self,
        title: Option<&str>,
        sender: Sender<String>,
        input_type: UserInputType,
        choices: Vec<String>,
    ) {
        let dialog = Dialog::with_buttons(
            title,
            Some(self),
            DialogFlags::MODAL,
            &[("Cancel", ResponseType::Cancel), ("OK", ResponseType::Ok)],
        );
        dialog.style_context().add_class("user-input");
        dialog.content_area().add(&Label::new(title));
        let mut entry = None;
        let mut list_box = None;
        match input_type {
            UserInputType::Text => {
                entry = Some(Entry::new());
                dialog.content_area().add(entry.as_ref().unwrap());
            }
            UserInputType::Password => {
                entry = Some(Entry::new());
                entry
                    .as_ref()
                    .unwrap()
                    .set_input_purpose(gtk::InputPurpose::Password);
                dialog.content_area().add(entry.as_ref().unwrap());
            }
            UserInputType::Choice => {
                list_box = Some(ListBox::new());
                list_box
                    .as_ref()
                    .unwrap()
                    .set_selection_mode(gtk::SelectionMode::Single);
                for choice in choices {
                    println!("{}", choice);
                    list_box.as_ref().unwrap().add(&Label::new(Some(&choice)));
                }
                dialog.content_area().add(list_box.as_ref().unwrap());
            }
        }
        dialog.show_all();

        dialog.connect_key_press_event(|dialog, key| {
            match key.keyval() {
                gtk::gdk::keys::constants::Escape => {
                    dialog.close();
                }
                gtk::gdk::keys::constants::Return => dialog.response(ResponseType::Ok),
                _ => {}
            };
            Inhibit(false)
        });

        dialog.connect_response(move |dialog, response_type| {
            match response_type {
                ResponseType::Ok => {
                    let user_input = match &entry {
                        Some(entry) => entry.text().to_string(),
                        None => list_box
                            .as_ref()
                            .unwrap()
                            .selected_row()
                            .as_ref()
                            .map(|row| row.child().unwrap().property::<String>("label"))
                            .unwrap_or(String::new()),
                    };
                    sender.send(user_input).unwrap();
                }
                _ => {
                    let _ = sender.send(String::new());
                }
            }
            dialog.close();
        });
    }
}
