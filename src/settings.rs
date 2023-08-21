mod cache_entry_row;
use std::fs::File;
use std::process::ChildStdin;

use flate2::read::GzDecoder;
use gtk::glib::{clone, user_data_dir};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::traits::{ButtonExt, ContainerExt, DialogExt, FileChooserExt, GtkWindowExt, WidgetExt};
use gtk::{gio, FileChooserAction, FileChooserDialog, Label, ResponseType};
use gtk::{glib, FileFilter};

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
        let dialog: gtk::Dialog = gtk::Dialog::builder()
            .transient_for(window)
            .modal(true)
            .title("Settings")
            .build();
        let content: Self = gtk::glib::Object::builder().build();
        content.plugin_list_fill(std::ops::Deref::deref(&window.imp().plugins.borrow()));
        dialog.content_area().add(&content);

        content
            .imp()
            .button_install
            .connect_clicked(clone!(@weak dialog => move |_| {
                Self::plugin_install(&dialog);
            }));

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

        dialog.connect_delete_event(move |_, _| {
            content.close();
            gtk::Inhibit(false)
        });

        dialog
    }

    fn plugin_list_fill(&self, plugins: &Vec<(ChildStdin, String, bool)>) {
        self.imp().listbox_installed.children().clear();
        for plugin in plugins.iter() {
            self.imp()
                .listbox_installed
                .add(&Label::new(Some(&plugin.1)));
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

    fn close(&self) {
        println!("Closing window");
    }
}
