use gtk::subclass::prelude::*;
use gtk::{glib, Button, ListBox, ScrolledWindow};
use gtk::{prelude::*, CompositeTemplate};

#[derive(Debug, Default, CompositeTemplate)]
#[template(file = "settings_window.ui")]
pub struct SettingsWindow {
    #[template_child]
    pub notebook: TemplateChild<gtk::Notebook>,
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
    #[template_child]
    pub checkbutton_posters: TemplateChild<gtk::CheckButton>,
    #[template_child]
    pub combobox_poster_size: TemplateChild<gtk::ComboBoxText>,
    #[template_child]
    pub checkbutton_backdrops: TemplateChild<gtk::CheckButton>,
    #[template_child]
    pub combobox_backdrop_size: TemplateChild<gtk::ComboBoxText>,

    #[template_child]
    pub listbox_args: TemplateChild<ListBox>,
    #[template_child]
    pub scrolledwindow_args: TemplateChild<ScrolledWindow>,
    #[template_child]
    pub entry_arg: TemplateChild<gtk::Entry>,
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
