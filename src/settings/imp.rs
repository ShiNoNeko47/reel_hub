use gtk::subclass::prelude::*;
use gtk::{glib, Button, ListBox};
use gtk::{prelude::*, CompositeTemplate};

#[derive(Debug, Default, CompositeTemplate)]
#[template(file = "settings_window.ui")]
pub struct SettingsWindow {
    #[template_child]
    pub button_install: TemplateChild<Button>,
    #[template_child]
    pub listbox_plugins: TemplateChild<ListBox>,
    #[template_child]
    pub listbox_cache: TemplateChild<ListBox>,
    #[template_child]
    pub button_clear: TemplateChild<Button>,

    #[template_child]
    pub revealer_images: TemplateChild<gtk::Revealer>,
    #[template_child]
    pub switch_images: TemplateChild<gtk::Switch>,
}

#[gtk::glib::object_subclass]
impl ObjectSubclass for SettingsWindow {
    const NAME: &'static str = "SettingsWindow";
    type Type = super::SettingsWindow;
    type ParentType = gtk::Box;

    fn class_init(klass: &mut Self::Class) {
        Self::bind_template(klass);
    }

    fn instance_init(obj: &gtk::glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for SettingsWindow {
    fn constructed(&self) {
        self.parent_constructed();
    }
}
impl WidgetImpl for SettingsWindow {}
impl ContainerImpl for SettingsWindow {}
impl BinImpl for SettingsWindow {}
impl BoxImpl for SettingsWindow {}
