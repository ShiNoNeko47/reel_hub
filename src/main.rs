use gtk::{
    glib::ExitCode, prelude::*, Box, Button, Dialog, Label, ListBox, Orientation, Picture,
    ResponseType, ScrolledWindow, Window,
};
use libadwaita::Application;
use regex::Regex;
use std::{ffi::OsStr, process::Command};
use walkdir::WalkDir;

fn main() -> ExitCode {
    let app = Application::builder()
        .application_id("com.gtk_rs.movies")
        .build();
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let hbox = Box::new(Orientation::Horizontal, 0);
    let list_box = ListBox::new();
    hbox.set_halign(gtk::Align::End);
    // hbox.set_valign(gtk::Align::Center);

    // let pixbuf = Pixbuf::from_file("RyuuguuRena.png").unwrap();
    let poster = Picture::new();
    hbox.append(&poster);

    list_box.set_margin_end(10);
    list_box.set_margin_start(10);
    list_box.set_margin_top(10);
    list_box.set_margin_bottom(10);

    let scrolled_window = ScrolledWindow::builder()
        .child(&list_box)
        .has_frame(true)
        .build();
    scrolled_window.set_hscrollbar_policy(gtk::PolicyType::Never);

    hbox.append(&scrolled_window);

    scrolled_window.set_margin_end(10);
    scrolled_window.set_margin_top(10);
    scrolled_window.set_margin_bottom(10);
    scrolled_window.set_margin_start(10);

    let main_window = Window::builder()
        .application(app)
        .title("movies")
        .child(&hbox)
        .build();

    for file in WalkDir::new("/home/nikola/Media/befafd9f-f32e-4121-978d-5abfe9b6bf6c/movies/")
        .into_iter()
        .filter_map(|file| file.ok())
    {
        if file.path().extension() == Some(OsStr::new("mp4")) {
            let movie_name: String = file.file_name().to_str().unwrap().to_string();

            let dialog = Dialog::builder().transient_for(&main_window).build();
            dialog.add_buttons(&[("Yes", ResponseType::Accept), ("No", ResponseType::Reject)]);
            dialog
                .content_area()
                .append(&Label::new(Some("Continue watching?")));
            dialog.content_area().set_margin_end(10);
            dialog.content_area().set_margin_start(10);
            dialog.content_area().set_margin_top(10);
            dialog.content_area().set_margin_bottom(10);
            dialog.connect_response(move |dialog, resp| {
                match resp {
                    ResponseType::Accept => {
                        play_movie(file.path().to_str().unwrap().to_string(), false)
                    }
                    ResponseType::Reject => {
                        play_movie(file.path().to_str().unwrap().to_string(), true)
                    }
                    _ => {}
                };
                dialog.hide();
            });

            let button = Button::builder().label(name_parse(&movie_name)).build();
            button.connect_clicked(move |_| {
                dialog.show();
            });

            list_box.append(&button);
            button.parent().unwrap().set_focusable(false);
        }
    }
    // main_window.connect_notify::<_>(Some("default-width"), |window, _| {
    //     window.set_default_width(window.width());
    //     window.child().unwrap().set_margin_start(window.width() / 3);
    // });
    main_window.present();
}

fn name_parse(name: &str) -> String {
    let re = Regex::new(r"^(.*)[\.| ]([0-9]{4})\.[\.|A-Z]*[0-9]+p\..*mp4").unwrap();
    let binding = re.captures(name);
    let name: String = match &binding {
        Some(expr) => format!("{} ({})", &expr[1], &expr[2]),
        None => name.to_string().replace(".mp4", ""),
    };
    name.replace(".", " ")
}

fn play_movie(path: String, from_start: bool) {
    Command::new("mpv")
        .arg(OsStr::new(&path))
        .arg("--save-position-on-quit")
        // .arg("--write-filename-in-watch-later-config")
        .arg(if from_start { "--start=0%" } else { "" })
        .output()
        .expect("Movie failed to play");
}
