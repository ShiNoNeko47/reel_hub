use gtk::{
    glib::ExitCode, prelude::*, Application, Box, Button, Orientation, ScrolledWindow, Window,
};
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
    let content = Box::new(Orientation::Vertical, 5);

    content.set_margin_end(10);
    content.set_margin_start(10);
    content.set_margin_top(10);
    content.set_margin_bottom(10);

    let scrolled_window = ScrolledWindow::builder()
        .child(&content)
        .has_frame(true)
        .build();

    scrolled_window.set_margin_end(10);
    scrolled_window.set_margin_start(10);
    scrolled_window.set_margin_top(10);
    scrolled_window.set_margin_bottom(10);

    let main_window = Window::builder()
        .application(app)
        .title("movies")
        .child(&scrolled_window)
        .build();

    for file in WalkDir::new("/home/nikola/Media/befafd9f-f32e-4121-978d-5abfe9b6bf6c/movies/")
        .into_iter()
        .filter_map(|file| file.ok())
    {
        if file.path().extension() == Some(OsStr::new("mp4")) {
            let movie_name: &str = file.file_name().to_str().unwrap();

            let button = Button::builder().label(name_parse(movie_name)).build();
            button.connect_clicked(move |_| {
                Command::new("mpv")
                    .arg(OsStr::new(file.path().to_str().unwrap()))
                    .spawn()
                    .expect("error");
            });
            content.append(&button);
        }
    }
    main_window.connect_notify::<_>(Some("default-width"), |window, _| {
        window.set_default_width(window.width());
        window.child().unwrap().set_margin_start(window.width() / 3);
    });

    main_window.present();
}

fn name_parse(name: &str) -> String {
    let re = Regex::new(r"^(.*)[\.| ]([0-9]{4})\.[\.|A-Z]*[0-9]+p\..*mp4").unwrap();
    let binding = re.captures(name);
    let name: String = match &binding {
        Some(expr) => format!("{} ({})", &expr[1], &expr[2]),
        None => name.to_string(),
    };
    name.replace(".", " ")
}
