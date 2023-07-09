use gtk::subclass::prelude::*;
use gtk::{glib, FileChooserWidget};
use gtk::{prelude::*, CompositeTemplate};

#[derive(Debug, Default, CompositeTemplate)]
#[template(file = "add_window.ui")]
pub struct Window {
    #[template_child]
    filechooser: TemplateChild<FileChooserWidget>,
}

impl Window {
}

#[glib::object_subclass]
impl ObjectSubclass for Window {
    const NAME: &'static str = "AddWindow";
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
