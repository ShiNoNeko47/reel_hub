use glib::{clone, user_data_dir, MainContext, PRIORITY_DEFAULT};
use gtk::{
    gdk_pixbuf::Pixbuf,
    gio::{Cancellable, MemoryInputStream},
    glib::ExitCode,
    prelude::*,
    Box, Button, Label, ListBox, Orientation, Picture, ScrolledWindow, Window,
};
use libadwaita::Application;
use regex::Regex;
use std::{ffi::OsStr, fs, path::Path, process::Command, thread};
use std::{
    io::{BufReader, Read, Write},
    path::PathBuf,
};
use walkdir::WalkDir;

fn main() -> ExitCode {
    let app = Application::builder()
        // .application_id("com.gtk_rs.movies")
        .build();
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let hbox = Box::new(Orientation::Horizontal, 0);
    let main_window = Window::builder()
        .application(app)
        .title("movies")
        .child(&hbox)
        .build();

    let walkdir = WalkDir::new(user_dir(user_data_dir())).follow_links(true);
    let mut files = walkdir
        .sort_by(|a, b| {
            a.file_name()
                .to_str()
                .unwrap()
                .cmp(&b.file_name().to_str().unwrap())
        })
        .into_iter()
        .filter_map(|file| file.ok())
        .filter_map(|file| {
            if file.path().extension() == Some(OsStr::new("mp4")) {
                Some(file)
            } else {
                None
            }
        })
        .peekable();

    if files.peek().is_none() {
        hbox.set_halign(gtk::Align::Center);
        hbox.append(&Label::new(Some(&format!(
            "To get started add movies to {} or make a symlink to a directory that contains movies",
            user_dir(user_data_dir()).to_str().unwrap()
        ))));
        main_window.present();
        return;
    }

    let info = Box::new(Orientation::Vertical, 5);
    info.set_halign(gtk::Align::Center);
    info.set_valign(gtk::Align::End);
    let poster = Picture::new();
    info.append(&poster);
    info.set_hexpand(true);
    info.set_halign(gtk::Align::Fill);

    hbox.append(&info);

    let list_box = ListBox::new();
    list_box.set_margin_end(10);
    list_box.set_margin_start(10);
    list_box.set_margin_top(10);
    list_box.set_margin_bottom(10);
    list_box.set_selection_mode(gtk::SelectionMode::None);

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

    for file in files {
        let movie_name = name_parse(file.file_name().to_str().unwrap().to_string());

        let button = Button::builder()
            .label(movie_name.0.replace(".", " "))
            .build();
        let (sender, reciever) = MainContext::channel(PRIORITY_DEFAULT);

        button.connect_clicked(clone!(@weak poster, @weak info => move |_| {
                let sender = sender.clone();
                let path = file.path().to_str().unwrap().to_string();
                let movie_name = movie_name.clone();
                thread::spawn(move ||{
                    let data = movie_data(&movie_name.0.replace(".", " "), &movie_name.1);
                    if Path::new(&format!("./src/pictures/cache/{}", data["poster_path"].as_str().unwrap())).exists() {
                        let file = BufReader::new(fs::File::open(format!("./src/pictures/cache/{}", data["poster_path"].as_str().unwrap())).unwrap());
                        sender.send(Some(file.bytes().map(|a| a.unwrap()).collect())).expect("Couldn't send");
                    }
                    else {
                        sender.send(None).expect("Couldn't send");
                        let result = reqwest::blocking::get(format!("https://image.tmdb.org/t/p/w185/{}", data["poster_path"].as_str().unwrap())).unwrap().bytes().unwrap().to_vec();
                        let bytes = glib::Bytes::from(&result.to_vec());
                        sender.send(Some(bytes.to_vec())).expect("Couldn't send");
                        let mut file = fs::File::create(format!("./src/pictures/cache{}", data["poster_path"].as_str().unwrap())).expect("Couldn't create file");
                        file.write(&bytes.to_vec()).expect("Couldn't write to file");
                    }
                });
                if info.last_child() != info.first_child() {
                    info.remove(&info.last_child().unwrap());
                }
                let play_button = Button::builder().label("Play").build();
                play_button.connect_clicked(move |_| {
                    play_movie(path.clone(), false);
                });
                info.append(&play_button);
            }));
        reciever.attach(
            None,
            clone!(@weak poster => @default-return Continue(false), move |bytes| {
                match bytes {
                    None => {
                        let _ = &poster.set_filename(Some("./src/pictures/Loading_dark.png"));
                        Continue(true)
                    },
                    Some(bytes) => {
                        let stream = MemoryInputStream::from_bytes(&glib::Bytes::from(&bytes));
                        let pixbuf = Pixbuf::from_stream(&stream, Cancellable::NONE).unwrap();
                        let _ = &poster.set_pixbuf(Some(&pixbuf));
                        Continue(true)
                    },
                }
            }),
        );

        list_box.append(&button);
    }
    main_window.present();
}

fn movie_data(name: &String, year: &String) -> serde_json::Value {
    let data = reqwest::blocking::get(format!(
        "https://api.themoviedb.org/3/search/movie?query={}&year={}&api_key={}",
        name, year, "f090bb54758cabf231fb605d3e3e0468"
    ))
    .unwrap()
    .text()
    .unwrap()
    .to_string();
    let results: serde_json::Value = serde_json::from_str(&data).unwrap();
    let mut movie_data = &results["results"][0];
    for result in results["results"].as_array().unwrap() {
        let title = result["title"].as_str().unwrap().to_string();
        let release_date = result["release_date"].as_str().unwrap().to_string();
        if title == name.to_string() && release_date.contains(year) {
            movie_data = result;
            break;
        }
    }
    movie_data.clone()
}

fn user_dir(path: PathBuf) -> PathBuf {
    let mut path = path;
    path.push("movies");
    fs::create_dir_all(&path).expect("Couldn't create directory");
    path
}

fn name_parse(name: String) -> (String, String, String) {
    let re = Regex::new(r"^(.*)[\.| ]([0-9]{4})\.[\.|A-Z]*[0-9]+p\..*mp4").unwrap();
    let binding = re.captures(&name);
    match &binding {
        Some(expr) => (
            expr[1].to_string(),
            expr[2].to_string(),
            format!("{}{}", &expr[1], &expr[2]),
        ),
        None => ("".to_string(), "".to_string(), "".to_string()),
    }
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
