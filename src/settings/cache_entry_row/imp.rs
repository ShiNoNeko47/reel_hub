use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

#[derive(Default, Debug, CompositeTemplate)]
#[template(file = "cache_entry_row.ui")]
pub struct CacheEntryRow {
    #[template_child]
    pub show_entry_button: TemplateChild<gtk::Button>,
    #[template_child]
    pub remove_entry_button: TemplateChild<gtk::Button>,
    #[template_child]
    pub cache_content: TemplateChild<gtk::Label>,
    #[template_child]
    pub revealer: TemplateChild<gtk::Revealer>,

    pub content_shown: RefCell<Rc<bool>>,
}

#[glib::object_subclass]
impl ObjectSubclass for CacheEntryRow {
    const NAME: &'static str = "CacheEntryRow";
    type Type = super::CacheEntryRow;
    type ParentType = gtk::Box;

    fn class_init(klass: &mut Self::Class) {
        Self::bind_template(klass);
    }

    fn instance_init(obj: &gtk::glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for CacheEntryRow {
    fn constructed(&self) {
        self.parent_constructed();
    }
}
impl WidgetImpl for CacheEntryRow {}
impl ContainerImpl for CacheEntryRow {}
impl BoxImpl for CacheEntryRow {}
