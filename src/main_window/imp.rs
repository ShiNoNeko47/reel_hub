use std::cell::{Cell, RefCell};
use std::fs::File;
use std::ops::Deref;
use std::path::PathBuf;
use std::process::ChildStdin;
use std::rc::Rc;

use crate::movie::{self, Movie, MovieCache, MovieData};
use crate::settings;
use crate::utils;
use gtk::glib::{clone, user_cache_dir, user_config_dir, Priority};
use gtk::subclass::prelude::*;
use gtk::{glib, Label, ListBox, Revealer, ScrolledWindow};
use gtk::{prelude::*, Button, CompositeTemplate, Image};

use crate::res;

#[derive(Debug, Default, CompositeTemplate)]
#[template(file = "main_window.ui")]
pub struct Window {
    #[template_child]
    pub poster: TemplateChild<Image>,
    #[template_child]
    pub backdrop: TemplateChild<Image>,
    #[template_child]
    pub backdrop_container: TemplateChild<ScrolledWindow>,
    #[template_child]
    pub play_button: TemplateChild<Button>,
    #[template_child]
    pub revealer: TemplateChild<Revealer>,
    #[template_child]
    pub list_box: TemplateChild<ListBox>,

    #[template_child]
    pub title: TemplateChild<Label>,
    #[template_child]
    pub original_title: TemplateChild<Label>,
    #[template_child]
    pub original_language: TemplateChild<Label>,
    #[template_child]
    pub overview: TemplateChild<Label>,
    #[template_child]
    pub vote_average: TemplateChild<Label>,
    #[template_child]
    pub vote_count: TemplateChild<Label>,
    #[template_child]
    pub release_date: TemplateChild<Label>,
    #[template_child]
    pub genres: TemplateChild<Label>,
    #[template_child]
    pub duration: TemplateChild<Label>,

    #[template_child]
    pub status_label: TemplateChild<Label>,
    #[template_child]
    pub add_button: TemplateChild<Button>,
    #[template_child]
    pub browse_button: TemplateChild<Button>,

    pub buttons: Rc<RefCell<Vec<Button>>>,
    pub button_selected: Rc<Cell<usize>>,

    pub movies: Rc<RefCell<Vec<Movie>>>,
    pub movies_len: Rc<Cell<usize>>,
    pub movie_selected: Rc<Cell<Option<usize>>>,
    pub movie_selected_tmp: Rc<Cell<Option<usize>>>,

    pub cache: Rc<RefCell<Vec<MovieCache>>>,

    pub dir_watcher: Rc<RefCell<Option<notify::RecommendedWatcher>>>,
    pub plugins: RefCell<Vec<(ChildStdin, String, bool)>>,

    pub settings: Rc<RefCell<settings::Settings>>,
}

impl Window {
    pub fn movie_select(&self, movie: Option<usize>) {
        self.movie_selected.replace(movie);
        match movie {
            Some(movie) => {
                let data = self.movies.borrow()[movie].data.clone();
                if self.movies.borrow()[movie].data.is_some() {
                    self.update_cache();
                }
                self.display_data(
                    data,
                    Some(&self.movies.borrow()[movie].name),
                    self.movies.borrow()[movie].duration.unwrap_or(0),
                );
                self.play_button.deref().show();
            }
            None => {
                self.display_data(None, None, 0);
                self.play_button.hide();
            }
        }
    }

    fn display_data(&self, data: Option<MovieData>, name: Option<&str>, duration: u32) {
        match data {
            None => {
                if let Some(name) = name {
                    self.poster
                        .deref()
                        .set_pixbuf(Some(&res::loading(&movie::ImageType::Poster)));
                    self.backdrop
                        .deref()
                        .set_pixbuf(Some(&res::loading(&movie::ImageType::Backdrop)));
                    self.title
                        .deref()
                        .set_label(&format!("<b>Title:</b> {name}"));
                } else {
                    self.poster.deref().set_pixbuf(None);
                    self.backdrop.deref().set_pixbuf(None);
                    self.title.deref().set_label("");
                }
                self.original_title.deref().set_label("");
                self.original_language.deref().set_label("");
                self.overview.deref().set_label("");
                self.vote_average.deref().set_label("");
                self.vote_count.deref().set_label("");
                self.release_date.deref().set_label("");
                self.genres.deref().set_label("");
                self.duration.deref().set_label("");
            }
            Some(data) => {
                self.title
                    .deref()
                    .set_label(&format!("<b>Title:</b> {}", data.title));
                self.original_title
                    .deref()
                    .set_label(&format!("<b>Original Title:</b> {}", data.original_title));
                self.original_language.deref().set_label(&format!(
                    "<b>Original Language:</b> {}",
                    data.original_language
                ));
                self.overview
                    .deref()
                    .set_label(&format!("<b>Overview:</b> {}", data.overview));
                self.vote_average.deref().set_label(&format!(
                    "<b>Vote Average:</b> {}",
                    data.vote_average.to_string()
                ));
                self.vote_count.deref().set_label(&format!(
                    "<b>Vote Count:</b> {}",
                    data.vote_count.to_string()
                ));
                self.release_date
                    .deref()
                    .set_label(&format!("<b>Release Date:</b> {}", data.release_date));
                self.genres
                    .deref()
                    .set_label(&format!("<b>Genres:</b> {}", data.genres.join(", ")));
                if duration > 0 {
                    self.duration.deref().set_label(&format!(
                        "<b>Duration:</b> {}:{:02}:{:02}",
                        duration / 3600,
                        duration / 60 % 60,
                        duration % 60
                    ));
                } else {
                    self.duration.deref().set_label("");
                }

                if data.poster_path != "".to_string() {
                    let mut image_path: Vec<String> =
                        data.poster_path.split("/").map(|x| x.to_string()).collect();
                    if let Some(file_name) = image_path.last_mut() {
                        *file_name = format!("w{}/{file_name}", self.settings.borrow().poster_w);
                    }
                    let image_path = image_path.join("/");
                    self.display_image(image_path, movie::ImageType::Poster);
                } else {
                    self.poster.deref().set_pixbuf(None);
                }

                if data.backdrop_path != "".to_string() {
                    let mut image_path: Vec<String> = data
                        .backdrop_path
                        .split("/")
                        .map(|x| x.to_string())
                        .collect();
                    if let Some(file_name) = image_path.last_mut() {
                        *file_name = format!("w{}/{file_name}", self.settings.borrow().backdrop_w);
                    }
                    let image_path = image_path.join("/");
                    self.display_image(image_path, movie::ImageType::Backdrop);
                } else {
                    self.backdrop.deref().set_pixbuf(None);
                }
            }
        }
    }
    fn display_image(&self, image_path: String, image_type: movie::ImageType) {
        let image_widget = match image_type {
            movie::ImageType::Poster => &self.poster,
            movie::ImageType::Backdrop => &self.backdrop,
        };
        let image_file_path;
        if PathBuf::from(&image_path).is_file() {
            image_file_path = image_path.clone();
        } else if image_path.split("/").collect::<Vec<&str>>().len() > 3 {
            image_widget.deref().set_pixbuf(None);
            return;
        } else {
            image_file_path = format!("{}{}", utils::user_dir(user_cache_dir()), image_path);
        }

        let (sender, receiver) =
            gtk::glib::MainContext::channel::<(PathBuf, movie::ImageType)>(Priority::default());
        match File::open(&image_file_path).map(|file| file.metadata().unwrap().len()) {
            Ok(0) | Err(_) => {
                image_widget
                    .deref()
                    .set_pixbuf(Some(&res::loading(&image_type)));
                std::thread::spawn(move || Movie::fetch_image(image_path, image_type, sender));
            }
            Ok(_) => {
                image_widget.deref().set_file(Some(&image_file_path));
            }
        }
        let movie_selected = self.movie_selected.get();
        receiver.attach(
            None,
            clone!(@weak self as window => @default-return Continue(false), move |(path, image_type)| {
                if movie_selected != window.movie_selected.get() {
                    println!("{:?} {:?}", movie_selected, window.movie_selected.get());
                    return Continue(true);
                }
                match image_type {
                    movie::ImageType::Poster => {
                        window.poster.deref().set_file(path.to_str());
                    }
                    movie::ImageType::Backdrop => {
                        window.backdrop.deref().set_file(path.to_str());
                    }
                }
                Continue(true)
            }),
        );
    }
    pub fn update_cache(&self) {
        for movie in self.movies.borrow_mut().iter_mut() {
            if movie.data.is_none() {
                continue;
            }
            let pos = self.cache.borrow_mut().iter_mut().position(|cache| {
                cache.file_name
                    == PathBuf::from(&movie.file)
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
            });
            let mut duration = movie.duration.unwrap_or(0);
            if duration == 0
                && !movie.name.starts_with("~ ")
                && PathBuf::from(movie.file.clone()).is_file()
            {
                println!("Duration is 0 for {}", movie.file);
                duration = match ffprobe::ffprobe(movie.file.clone()) {
                    Ok(info) => {
                        let duration = info.format.duration.unwrap().parse::<f32>().unwrap() as u32;
                        movie.duration.replace(duration);
                        duration
                    }
                    Err(_) => 0,
                };
            }
            let cache = MovieCache {
                file_name: PathBuf::from(&movie.file)
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
                duration,
                done: movie.done,
                data: movie.data.clone().unwrap(),
            };
            if let Some(pos) = pos {
                self.cache.borrow_mut()[pos] = cache;
            } else {
                self.cache.borrow_mut().push(cache);
            }
        }
    }

    pub fn store_cache(&self) {
        let mut path = utils::user_dir(user_cache_dir());
        path.push_str("/movie_data.json");

        let file = std::fs::File::create(path).expect("Could not create file");
        serde_json::to_writer(file, &self.cache.borrow().to_vec())
            .expect("Could not write to file");
    }

    pub fn store_settings(&self) {
        let mut path = utils::user_dir(user_config_dir());
        path.push_str("/settings.json");

        let file = std::fs::File::create(path).expect("Could not create file");
        serde_json::to_writer(file, &self.settings.take()).expect("Could not write to file");
    }

    pub fn apply_settings(&self) {
        let settings = self.settings.borrow();
        self.poster
            .set_visible(settings.images_enabled && settings.poster_enabled);
        self.poster.set_width_request(settings.poster_w as i32);
        self.backdrop_container
            .set_visible(settings.images_enabled && settings.backdrop_enabled);
        self.movie_select(self.movie_selected.get());
    }
}

#[gtk::glib::object_subclass]
impl ObjectSubclass for Window {
    const NAME: &'static str = "MoviesWindow";
    type Type = super::Window;
    type ParentType = gtk::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        Self::bind_template(klass);
    }

    fn instance_init(obj: &gtk::glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for Window {
    fn constructed(&self) {
        self.parent_constructed();
    }
}
impl WidgetImpl for Window {}
impl ContainerImpl for Window {}
impl BinImpl for Window {}
impl WindowImpl for Window {}
impl ApplicationWindowImpl for Window {}
