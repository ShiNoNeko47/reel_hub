use std::cell::{Cell, RefCell};
use std::fs::File;
use std::ops::Deref;
use std::path::PathBuf;
use std::process::ChildStdin;
use std::rc::Rc;

use gdk_pixbuf::Pixbuf;
use glib::{clone, user_cache_dir, Priority};
use gtk::subclass::prelude::*;
use gtk::{glib, Label, ListBox, Revealer, ScrolledWindow};
use gtk::{prelude::*, Button, CompositeTemplate, Image};
use reel_hub::movie::{Movie, MovieCache, MovieData};
use reel_hub::utils;

use reel_hub::res;

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

    pub cache: Rc<RefCell<Vec<MovieCache>>>,

    pub dir_watcher: Rc<RefCell<Option<notify::RecommendedWatcher>>>,
    pub plugins: RefCell<Vec<ChildStdin>>,
}

impl Window {
    pub fn movie_select(&self, movie: Option<usize>) {
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
        self.movie_selected.replace(movie);
    }

    fn display_data(&self, data: Option<MovieData>, name: Option<&str>, duration: u32) {
        match data {
            None => {
                if let Some(name) = name {
                    self.poster
                        .deref()
                        .set_pixbuf(Some(&res::loading(&reel_hub::movie::ImageType::Poster)));
                    self.backdrop
                        .deref()
                        .set_pixbuf(Some(&res::loading(&reel_hub::movie::ImageType::Backdrop)));
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
                    self.display_image(
                        data.poster_path.clone(),
                        reel_hub::movie::ImageType::Poster,
                    );
                } else {
                    self.poster.deref().set_pixbuf(None);
                }

                if data.backdrop_path != "".to_string() {
                    self.display_image(
                        data.backdrop_path.clone(),
                        reel_hub::movie::ImageType::Backdrop,
                    );
                } else {
                    self.backdrop.deref().set_pixbuf(None);
                }
            }
        }
    }
    fn display_image(&self, image_path: String, image_type: reel_hub::movie::ImageType) {
        let image_file_path = format!("{}{}", utils::user_dir(user_cache_dir()), image_path);

        let (sender, receiver) = glib::MainContext::channel::<(PathBuf, reel_hub::movie::ImageType)>(
            Priority::default(),
        );
        let image_widget = match image_type {
            reel_hub::movie::ImageType::Poster => &self.poster,
            reel_hub::movie::ImageType::Backdrop => &self.backdrop,
        };
        match File::open(&image_file_path) {
            Ok(_) => {
                if let Ok(pixbuf) = &Pixbuf::from_file(image_file_path) {
                    image_widget.deref().set_pixbuf(Some(pixbuf))
                };
            }
            Err(_) => {
                image_widget
                    .deref()
                    .set_pixbuf(Some(&res::loading(&image_type)));
                std::thread::spawn(move || Movie::fetch_image(image_path, image_type, sender));
            }
        }
        receiver.attach(
            None,
            clone!(@weak self as window => @default-return Continue(false), move |(path, image_type)| {
                match image_type {
                    reel_hub::movie::ImageType::Poster => {
                        window.poster.deref().set_pixbuf(Some(&Pixbuf::from_file(path).unwrap()));
                    }
                    reel_hub::movie::ImageType::Backdrop => {
                        window.backdrop.deref().set_pixbuf(Some(&Pixbuf::from_file(path).unwrap()));
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
                cache.file_name == movie.file.file_name().unwrap().to_str().unwrap()
            });
            let mut duration = movie.duration.unwrap_or(0);
            if duration == 0 && !movie.name.starts_with("~ ") {
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
                file_name: movie
                    .file
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
}

#[glib::object_subclass]
impl ObjectSubclass for Window {
    const NAME: &'static str = "MoviesWindow";
    type Type = super::Window;
    type ParentType = gtk::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        Self::bind_template(klass);
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
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
