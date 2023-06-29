use std::cell::{RefCell, Cell};
use std::rc::Rc;

use gtk::subclass::prelude::*;
use gtk::{glib, ListBox, Label};
use gtk::{prelude::*, Button, CompositeTemplate, Image};
use movies::movie::Movie;

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
