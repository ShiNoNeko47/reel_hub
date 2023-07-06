use std::cell::{RefCell, Cell};
use std::fs::File;
use std::ops::Deref;
use std::path::PathBuf;
use std::rc::Rc;

use gdk_pixbuf::Pixbuf;
use glib::{user_cache_dir, Priority, clone};
use gtk::subclass::prelude::*;
use gtk::{glib, ListBox, Label};
use gtk::{prelude::*, Button, CompositeTemplate, Image};
use movies::movie::{Movie, MovieCache, MovieData};

use crate::res;

#[derive(Debug, Default, CompositeTemplate)]
#[template(file = "window.ui")]
pub struct Window {
    #[template_child]
    pub poster: TemplateChild<Image>,
    #[template_child]
    pub play_button: TemplateChild<Button>,
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

    pub movies: Rc<RefCell<Vec<Movie>>>,
    pub movies_len: Rc<Cell<usize>>,
    pub movie_selected: Rc<Cell<Option<usize>>>,

    pub dir_watcher: Rc<RefCell<Option<notify::RecommendedWatcher>>>,
}

impl Window {
    pub fn movie_select(&self, movie: usize) {
        let data = self.movies.borrow()[movie].data.clone();

        self.movie_selected.replace(Some(movie));

        self.play_button.deref().set_label("  Play  ");
        self.play_button.deref().show();

        self.display_data(data, Some(&self.movies.borrow()[movie].name));
        self.cache();
    }

    fn display_data(&self, data: Option<MovieData>, name: Option<&str>) {
        match data {
            None => {
                if let Some(name) = name {
                    self.poster.deref().set_pixbuf(Some(&res::loading()));
                    self.title.deref().set_label(&format!("<b>Title:</b> {name}"));
                }
                self.original_title.deref().set_label("");
                self.original_language.deref().set_label("");
                self.overview.deref().set_label("");
                self.vote_average.deref().set_label("");
                self.vote_count.deref().set_label("");
                self.release_date.deref().set_label("");
            },
            Some(data) => {
                self.title.deref().set_label(&format!("<b>Title:</b> {}", data.title));
                self.original_title.deref().set_label(&format!("<b>Original Title:</b> {}", data.original_title));
                self.original_language.deref().set_label(&format!("<b>Original Language:</b> {}", data.original_language));
                self.overview.deref().set_label(&format!("<b>Overview:</b> {}", data.overview));
                self.vote_average.deref().set_label(&format!("<b>Vote Average:</b> {}", data.vote_average.to_string()));
                self.vote_count.deref().set_label(&format!("<b>Vote Count:</b> {}", data.vote_count.to_string()));
                self.release_date.deref().set_label(&format!("<b>Release Date:</b> {}", data.release_date));

                self.display_poster(data);
            }
        }
    }
    fn display_poster(&self, data: MovieData) {
        let poster_file_path = format!("{}{}", movies::utils::user_dir(user_cache_dir()), data.poster_path);

        let (sender, receiver) = glib::MainContext::channel::<PathBuf>(Priority::default());
        match File::open(&poster_file_path) {
            Ok(_) => {
                self.poster.deref().set_pixbuf(Some(&Pixbuf::from_file(poster_file_path).unwrap()));
            }
            Err(_) => {
                self.poster.deref().set_pixbuf(Some(&res::loading()));
                let poster_path = data.poster_path.clone();
                std::thread::spawn(move || {
                    Movie::fetch_poster(poster_path, sender)
                });
            }
        }
        receiver.attach(None, clone!(@weak self as window => @default-return Continue(false), move |path| {
            window.poster.deref().set_pixbuf(Some(&Pixbuf::from_file(path).unwrap()));
            Continue(true)
        }));
    }
    fn cache(&self) {
        let mut path = movies::utils::user_dir(user_cache_dir());
        path.push_str("/movie_data.json");

        let cache_data: Vec<MovieCache> = self.movies.borrow().iter().filter(|x| x.data.is_some()).map(|x| MovieCache{file: x.file.clone(), data: x.data.clone().unwrap()}).collect();

        let file = std::fs::File::create(path).expect("Could not create file");
        serde_json::to_writer(file, &cache_data).expect("Could not write to file");
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
