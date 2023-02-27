use glib::{clone, user_cache_dir, user_data_dir, MainContext, PRIORITY_DEFAULT};
use gtk::{
    gdk_pixbuf::Pixbuf,
    gio::{Cancellable, MemoryInputStream},
    glib::ExitCode,
    prelude::*,
    Box, Button, Label, ListBox, Orientation, Picture, ScrolledWindow, Widget, Window,
};
use libadwaita::Application;
use regex::Regex;
use std::{
    cell::RefCell,
    collections::HashMap,
    ffi::OsStr,
    fs,
    path::Path,
    process::{Command, Stdio},
    thread,
};
use std::{
    io::{BufReader, Read, Write},
    path::PathBuf,
};
use walkdir::WalkDir;

thread_local! {
    static MOVIE_DATA_CACHE: RefCell<HashMap<String, serde_json::Value>> = HashMap::new().into();
}

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
            user_dir(user_data_dir())
        ))));
        main_window.present();
        return;
    }

    let content = Box::new(Orientation::Vertical, 5);
    set_margins(10, &content);
    content.set_halign(gtk::Align::Center);
    content.set_valign(gtk::Align::End);

    let poster = Picture::new();
    poster.set_hexpand(true);
    content.append(&poster);

    let info = Box::new(Orientation::Vertical, 5);
    info.set_hexpand(true);
    info.set_halign(gtk::Align::Fill);
    content.append(&info);

    content.set_hexpand(true);
    content.set_halign(gtk::Align::Fill);

    hbox.append(&content);

    let list_box = ListBox::new();
    set_margins::<ListBox>(10, &list_box);
    list_box.set_selection_mode(gtk::SelectionMode::None);

    let scrolled_window = ScrolledWindow::builder()
        .child(&list_box)
        .has_frame(true)
        .build();
    scrolled_window.set_hscrollbar_policy(gtk::PolicyType::Never);

    hbox.append(&scrolled_window);

    set_margins(10, &scrolled_window);

    for file in files {
        let path = file.path().to_str().unwrap().to_string();
        let movie_name = name_parse(file.file_name().to_str().unwrap().to_string());

        let button = Button::builder()
            .label(movie_name.0.replace(".", " "))
            .build();
        let (sender, reciever) = MainContext::channel(PRIORITY_DEFAULT);

        button.connect_clicked(clone!(@weak poster, @weak info => move |_| {
            let sender = sender.clone();
            let movie_name = movie_name.clone();
            poster.hide();
            while info.last_child() != None {
                info.remove(&info.last_child().unwrap());
            }
            let data = MOVIE_DATA_CACHE.with(|it| {
                let movie_data_cache = it.borrow_mut();
                if movie_data_cache.contains_key(&path) {
                    Some(movie_data_cache[&path].clone())
                }
                else {
                    None
                }
            });
            match data {
                Some(data) => {
                        if Path::new(&format!(
                                "{}{}",
                                user_dir(user_cache_dir()),
                                data["poster_path"].as_str().unwrap()))
                            .exists() {
                            let file = BufReader::new(fs::File::open(format!(
                                        "{}{}",
                                        user_dir(user_cache_dir()),
                                        data["poster_path"].as_str().unwrap()))
                                .unwrap());
                            sender.send(
                                (Some(file.bytes().map(|a| a.unwrap()).collect()), Some(data.clone())))
                                .expect("Couldn't send");
                        }
                },
                None => {
                    thread::spawn(move ||{
                        sender.send((None, None)).expect("Couldn't send");
                        let data = movie_data(&movie_name.0.replace(".", " "), &movie_name.1);
                        if Path::new(&format!(
                                "{}{}",
                                user_dir(user_cache_dir()),
                                data["poster_path"].as_str().unwrap()))
                            .exists() {
                            let file = BufReader::new(fs::File::open(format!(
                                        "{}{}",
                                        user_dir(user_cache_dir()),
                                        data["poster_path"].as_str().unwrap()))
                                .unwrap());
                            sender.send(
                                (Some(file.bytes().map(|a| a.unwrap()).collect()), Some(data.clone())))
                                .expect("Couldn't send");
                        }
                        else {
                            sender.send((None, Some(data.clone()))).expect("Couldn't send");
                            let result = reqwest::blocking::get(format!(
                                    "https://image.tmdb.org/t/p/w185/{}",
                                    data["poster_path"].as_str().unwrap()))
                                .unwrap()
                                .bytes()
                                .unwrap()
                                .to_vec();
                            let bytes = glib::Bytes::from(&result.to_vec());
                            sender.send(
                                (Some(bytes.to_vec()), Some(data.clone())))
                                .expect("Couldn't send");
                            let mut file = fs::File::create(format!(
                                    "{}{}",
                                    user_dir(user_cache_dir()),
                                    data["poster_path"].as_str().unwrap()))
                                .expect("Couldn't create file");
                            file.write(&bytes.to_vec()).expect("Couldn't write to file");
                        }
                    });
                },
            }
        }));
        reciever.attach(
            None,
            clone!(@weak poster, @weak info => @default-return Continue(false), move |(bytes, data)| {
                let path = file.path().to_str().unwrap().to_string();
                let play_button = Button::builder().label("Play").build();
                play_button.connect_clicked(move |_| {
                    play_movie(&path, false);
                });
                match data {
                    Some(data) => {
                        if info.first_child() == info.last_child() && info.last_child() != None {
                            info.remove(&info.last_child().unwrap());
                        }
                        if info.last_child() == None {
                            show_info(&info, &data);
                            info.append(&play_button);
                        }

                        MOVIE_DATA_CACHE.with(|it| {
                            let mut movie_data_cache = it.borrow_mut();
                            movie_data_cache.insert(file.path().to_str().unwrap().to_string(), data);
                        });
                    },
                    None => {
                        info.append(&Label::builder()
                                    .label("<b>Loading...</b>")
                                    .use_markup(true)
                                    .build());
                    },
                }
                match bytes {
                    None => {
                        let bytes = glib::Bytes::from(include_bytes!("pictures/Loading_dark.png"));
                        let stream = MemoryInputStream::from_bytes(&bytes);
                        let pixbuf = Pixbuf::from_stream(&stream, Cancellable::NONE).unwrap();
                        let _ = &poster.set_pixbuf(Some(&pixbuf));
                        poster.show();
                    },
                    Some(bytes) => {
                        let stream = MemoryInputStream::from_bytes(&glib::Bytes::from(&bytes));
                        let pixbuf = Pixbuf::from_stream(&stream, Cancellable::NONE).unwrap();
                        let _ = &poster.set_pixbuf(Some(&pixbuf));
                        poster.show();
                    },
                }
                Continue(true)
            }),
        );

        list_box.append(&button);
    }
    main_window.present();
}

fn show_info(info: &Box, data: &serde_json::Value) {
    info.append(
        &Label::builder()
            .label(&format!(
                "<b>Original title:</b> {}",
                data["original_title"].as_str().unwrap()
            ))
            .use_markup(true)
            .build(),
    );
    info.append(
        &Label::builder()
            .label(&format!(
                "<b>Original language:</b> {}",
                data["original_language"].as_str().unwrap()
            ))
            .use_markup(true)
            .build(),
    );
    info.append(
        &Label::builder()
            .label(&format!(
                "<b>Overview:</b>\n {}",
                data["overview"].as_str().unwrap()
            ))
            .use_markup(true)
            .wrap(true)
            .justify(gtk::Justification::Center)
            .build(),
    );
    info.append(
        &Label::builder()
            .label(&format!(
                "<b>Vote average (tmdb):</b> {}",
                data["vote_average"].as_f64().unwrap()
            ))
            .use_markup(true)
            .build(),
    );
    info.append(
        &Label::builder()
            .label(&format!(
                "<b>Vote count (tmdb):</b> {}",
                data["vote_count"].as_f64().unwrap()
            ))
            .use_markup(true)
            .build(),
    );
    info.append(
        &Label::builder()
            .label(&format!(
                "<b>Release date:</b> {}",
                data["release_date"].as_str().unwrap()
            ))
            .use_markup(true)
            .build(),
    );
}

fn set_margins<T>(size: i32, widget: &T)
where
    T: IsA<Widget>,
{
    widget.set_margin_end(size);
    widget.set_margin_start(size);
    widget.set_margin_top(size);
    widget.set_margin_bottom(size);
}

fn movie_data(name: &String, year: &Option<String>) -> serde_json::Value {
    /*{adult: bool, backdrop_path: String, genre_ids: [i32], id: i32, original_language: String,
     * original_title: String, overview: String, popularity: f32, poster_path: String,
     * release_date: String, title: String, vote_average: f32, vote_count: i32}*/

    let year = match year {
        Some(year) => format!("&year={}", year),
        None => "".to_string(),
    };

    let data = reqwest::blocking::get(format!(
        "https://api.themoviedb.org/3/search/movie?query={}{}&api_key={}",
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
        if title == name.to_string() && release_date.contains(&year.replace("&year=", "")) {
            movie_data = result;
            break;
        }
    }
    movie_data.to_owned()
}

fn user_dir(path: PathBuf) -> String {
    let mut path = path;
    path.push("movies");
    fs::create_dir_all(&path).expect("Couldn't create directory");
    path.to_str().unwrap().to_string()
}

fn name_parse(name: String) -> (String, Option<String>, String) {
    let re = Regex::new(r"^(.*)[\.| ]([0-9]{4})\.[\.|A-Z]*[[0-9]+p]*.*mp4").unwrap();
    let binding = re.captures(&name);
    match &binding {
        Some(expr) => (
            expr[1].to_string(),
            Some(expr[2].to_string()),
            format!("{}{}", &expr[1], &expr[2]),
        ),
        None => (name.replace(".mp4", ""), None, "".to_string()),
    }
}

fn play_movie(path: &String, from_start: bool) {
    Command::new("mpv")
        .arg(OsStr::new(&path))
        .arg("--save-position-on-quit")
        // .arg("--write-filename-in-watch-later-config")
        .arg(if from_start { "--start=0%" } else { "" })
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Movie failed to play");
}
