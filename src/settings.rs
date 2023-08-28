mod cache_entry_row;
use std::fs::File;
use std::io::Write;

use flate2::read::GzDecoder;
use gtk::glib::{clone, user_data_dir};
use gtk::subclass::prelude::*;
use gtk::traits::{ButtonExt, ContainerExt, DialogExt, FileChooserExt, GtkWindowExt, WidgetExt};
use gtk::{gio, FileChooserAction, FileChooserDialog, Label, ResponseType};
use gtk::{glib, FileFilter};
use gtk::{prelude::*, Button};

use crate::main_window::Window;
use crate::utils;

mod imp;

gtk::glib::wrapper! {
    pub struct SettingsWindow(ObjectSubclass<imp::SettingsWindow>)
        @extends gtk::Box, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Buildable;
}

impl SettingsWindow {
    pub fn new(window: &Window) -> gtk::Dialog {
        window.window().unwrap().freeze_updates();
        let dialog: gtk::Dialog = gtk::Dialog::builder()
            .transient_for(window)
            .modal(true)
            .title("Settings")
            .build();
        let content: Self = gtk::glib::Object::builder().build();
        content.plugin_list_fill(&window);
        dialog.content_area().add(&content);

        content
            .imp()
            .button_install
            .connect_clicked(clone!(@weak dialog => move |_| {
                Self::plugin_install(&dialog);
            }));

        content.setup_settings(&window);

        for entry in window.imp().cache.borrow().iter() {
            let mut entry = entry.clone();
            entry.data.overview = "...".to_string();

            let row = cache_entry_row::CacheEntryRow::new(
                entry.data.title.clone(),
                format!("{:#?}", entry),
            );
            content.imp().listbox_cache.add(&row);
            let listbox_children = content.imp().listbox_cache.children();
            let listbox_row = listbox_children.last().unwrap();
            row.imp().remove_entry_button.connect_clicked(
                clone!(@weak content, @weak listbox_row, @weak window => move |_| {
                    content.imp().listbox_cache.remove(&listbox_row);
                    let position = window.imp().cache.borrow().iter().position(|item| item.file_name == entry.file_name).unwrap();
                    window.imp().cache.borrow_mut().remove(position);
                }),
            );
        }
        content.imp().button_clear.connect_clicked(
            clone!(@weak content, @weak window => move |_| {
                content.imp().listbox_cache.forall(|widget| content.imp().listbox_cache.remove(widget));
                window.imp().cache.borrow_mut().clear();
            }),
        );

        let starting_height = content.allocated_height();
        content.connect_size_allocate(move |content, rectangle| {
            if starting_height < rectangle.height() {
                content
                    .imp()
                    .scrolledwindow_args
                    .set_vscrollbar_policy(gtk::PolicyType::Automatic);
            }
        });

        dialog.connect_delete_event(
            clone!(@weak window => @default-return gtk::Inhibit(false), move |_, _| {
                content.close(&window);
                gtk::Inhibit(false)
            }),
        );

        dialog
    }
    fn setup_settings(&self, window: &Window) {
        let settings = window.imp().settings.borrow();
        self.imp().switch_images.set_active(settings.images_enabled);
        self.imp()
            .revealer_images
            .set_reveal_child(settings.images_enabled);
        self.imp()
            .checkbutton_posters
            .set_active(settings.poster_enabled);
        self.imp()
            .checkbutton_backdrops
            .set_active(settings.backdrop_enabled);
        self.imp()
            .combobox_poster_size
            .set_active_id(Some(&format!("w{}", settings.poster_w)));
        self.imp()
            .combobox_backdrop_size
            .set_active_id(Some(&format!("w{}", settings.backdrop_w)));
        for arg in settings.player_args.iter() {
            let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 10);
            let label = Label::builder()
                .label(arg)
                .halign(gtk::Align::Start)
                .build();
            let button = Button::with_label("Remove");
            hbox.add(&label);
            hbox.add(&button);
            self.imp().listbox_args.add(&hbox);
            let arg = arg.clone();
            button.connect_clicked(clone!(@weak self as content, @weak window => move |_| {
                content.imp().listbox_args.remove(&hbox.parent().unwrap());
                window.imp().settings.borrow_mut().player_args.retain(|item| item != &arg);
            }));
        }

        self.imp().switch_images.connect_changed_active(
            clone!(@weak self as content, @weak window => move |switch| {
                content
                    .imp()
                    .revealer_images
                    .set_reveal_child(switch.is_active());
                window.imp().settings.borrow_mut().images_enabled = switch.is_active();
                window.imp().apply_settings();
            }),
        );

        self.imp()
            .checkbutton_posters
            .connect_clicked(clone!(@weak window => move |checkbutton| {
                window.imp().settings.borrow_mut().poster_enabled = checkbutton.is_active();
                window.imp().apply_settings();
            }));

        self.imp().checkbutton_backdrops.connect_clicked(
            clone!(@weak window => move |checkbutton| {
                window.imp().settings.borrow_mut().backdrop_enabled = checkbutton.is_active();
                window.imp().apply_settings();
            }),
        );
        self.imp()
            .combobox_poster_size
            .connect_changed(
            clone!(@weak window => move |combobox| {
                window.imp().settings.borrow_mut().poster_w = combobox.active_id().unwrap().as_str()[1..].parse().unwrap();
                window.imp().apply_settings();
            })
        );
        self.imp()
            .combobox_backdrop_size
            .connect_changed(
            clone!(@weak window => move |combobox| {
                window.imp().settings.borrow_mut().backdrop_w = combobox.active_id().unwrap().as_str()[1..].parse().unwrap();
                window.imp().apply_settings();
            })
        );
        self.imp().entry_arg.connect_activate(
            clone!(@weak self as content, @weak window => move |entry| {
                let buffer = entry.buffer();
                let arg = buffer.text();
                if !arg.is_empty() {
                    buffer.set_text("");
                    if window.imp().settings.borrow().player_args.contains(&arg) {
                        return;
                    }
                    window
                        .imp()
                        .settings
                        .borrow_mut()
                        .player_args
                        .push(arg.to_string());
                    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 10);
                    let label = Label::builder()
                        .label(&arg)
                        .halign(gtk::Align::Start)
                        .build();
                    let button = Button::with_label("Remove");
                    hbox.add(&label);
                    hbox.add(&button);
                    content.imp().listbox_args.add(&hbox);
                    content.show_all();
                    let arg = arg.clone();
                    button.connect_clicked(clone!(@weak content, @weak window => move |_| {
                        content.imp().listbox_args.remove(&hbox.parent().unwrap());
                        window.imp().settings.borrow_mut().player_args.retain(|item| item != &arg);
                    }));
                }
            }),
        );
    }

    fn plugin_list_fill(&self, window: &Window) {
        for (i, plugin) in window.imp().plugins.borrow().iter().enumerate() {
            let label = Label::new(Some(
                &(plugin.1.clone() + if plugin.2 { " - Running..." } else { "" }),
            ));

            let stop_button = Button::with_label("Stop");
            stop_button.set_sensitive(plugin.2);
            stop_button.connect_clicked(clone!(@weak window, @weak label => move |button| {
                let _ = window.imp().plugins.borrow_mut()[i].0.write_all("0\n".as_bytes());
                label.set_label(&window.imp().plugins.borrow()[i].1);
                button.set_sensitive(false);
                let _ = window.imp().plugins.borrow_mut()[i].2 = false;
            }));

            let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 10);
            hbox.add(&label);
            hbox.add(&stop_button);
            self.imp().listbox_plugins.add(&hbox);
        }
    }

    pub fn plugin_install(dialog: &gtk::Dialog) {
        let filechooser = FileChooserDialog::with_buttons(
            Some("Install a Plugin"),
            Some(dialog),
            FileChooserAction::Open,
            &[("Open", ResponseType::Ok), ("Cancel", ResponseType::Cancel)],
        );
        filechooser.set_modal(true);
        filechooser.set_filter(&FileFilter::new());
        filechooser
            .filter()
            .unwrap()
            .add_mime_type("application/x-compressed-tar");
        filechooser.show_all();

        filechooser.connect_response(|dialog, response| {
            match response {
                ResponseType::Ok => {
                    let file = dialog.file();
                    let dst = utils::user_dir(user_data_dir()) + "/.plugins/";
                    let tar_gz = File::open(file.unwrap().path().unwrap());
                    if let Ok(tar_gz) = tar_gz {
                        let tar = GzDecoder::new(tar_gz);
                        let mut archive = tar::Archive::new(tar);
                        archive.unpack(dst).unwrap();
                    }
                }
                _ => {}
            };
            dialog.close();
        });
    }

    fn close(&self, window: &Window) {
        window.window().unwrap().thaw_updates();
        window.queue_allocate();
    }
}

#[derive(Debug)]
pub struct Settings {
    pub images_enabled: bool,
    pub poster_enabled: bool,
    pub poster_w: u32,
    pub backdrop_enabled: bool,
    pub backdrop_w: u32,
    pub player_args: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            images_enabled: true,
            poster_enabled: true,
            poster_w: 500,
            backdrop_enabled: true,
            backdrop_w: 780,
            player_args: vec![
                "--no-config".to_string(),
                "--save-position-on-quit".to_string(),
                format!(
                    "--watch-later-directory={}/.watch-later",
                    super::utils::user_dir(user_data_dir())
                ),
                "--fs".to_string(),
                "--ytdl-format=mp4".to_string(),
                "--watch-later-options=start".to_string(),
            ],
        }
    }
}
