// mod window;

use glib::{clone, user_data_dir, MainContext, PRIORITY_DEFAULT};
use gtk::gio::resources_register_include;
use gtk::Application;
use gtk::{
    gdk_pixbuf::Pixbuf,
    gio::{Cancellable, MemoryInputStream},
    glib::ExitCode,
    prelude::*,
    Box, Button, Image, Label, ListBox, Orientation, ScrolledWindow, Widget, Window,
};

use std::path::PathBuf;
use std::sync::Mutex;
use std::{fs, thread};

use movies::{
    detect,
    movie::{Movie, MovieData},
};

static LOADING_IMAGE_DARK: &[u8; 2904] = include_bytes!("pictures/Loading_dark.png");
static MOVIES: Mutex<Vec<Movie>> = Mutex::new(vec![]);

static MOVIE_SELECTED: Mutex<usize> = Mutex::new(0);

fn main() -> ExitCode {
    resources_register_include!("movies.gresource").expect("Failed loading resource");
    let app = Application::builder()
        // .application_id("com.gtk_rs.movies")
        .build();
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    // let window = window::Window::new(app);
    // window.show_all();
    let hbox: Box = Box::new(Orientation::Horizontal, 0);
    let main_window: Window = Window::builder()
        .application(app)
        .title("movies")
        .child(&hbox)
        .build();

    *MOVIES.lock().unwrap() = detect::get_movies(user_dir(user_data_dir()));

    if MOVIES.lock().unwrap().len() == 0 {
        hbox.set_halign(gtk::Align::Center);
        hbox.add(&Label::new(Some(&format!(
            "To get started add movies to {} or make a symlink to a directory that contains movies",
            user_dir(user_data_dir())
        ))));
        main_window.show_all();
        return;
    }

    let content: Box = Box::new(Orientation::Horizontal, 5);
    set_margins(10, &content);
    content.set_halign(gtk::Align::Start);
    content.set_valign(gtk::Align::Center);

    let poster: Image = Image::new();
    poster.set_hexpand(true);
    content.add(&poster);

    let info: Box = Box::new(Orientation::Vertical, 5);
    info.set_hexpand(true);
    info.set_halign(gtk::Align::Fill);
    info.set_valign(gtk::Align::Center);
    for _ in 0..7 {
        info.add(
            &Label::builder()
                .use_markup(true)
                .wrap(true)
                .justify(gtk::Justification::Center)
                .build(),
        );
    }
    content.add(&info);

    let play_button: Button = Button::builder().label("Play").build();
    play_button.connect_clicked(move |_| {
        MOVIES.lock().unwrap()[*MOVIE_SELECTED.lock().unwrap()].play(false)
    });
    play_button.set_sensitive(false);
    info.add(&play_button);

    content.set_hexpand(true);
    content.set_halign(gtk::Align::Fill);

    hbox.add(&content);

    let list_box: ListBox = ListBox::new();
    set_margins(10, &list_box);
    list_box.set_selection_mode(gtk::SelectionMode::None);

    let scrolled_window: ScrolledWindow = ScrolledWindow::builder()
        .child(&list_box)
        // .has_frame(true)
        .build();
    scrolled_window.set_hscrollbar_policy(gtk::PolicyType::Never);

    hbox.add(&scrolled_window);

    set_margins(10, &scrolled_window);

    let (sender, reciever) = MainContext::channel(PRIORITY_DEFAULT);

    let movies_length: usize = MOVIES.lock().unwrap().len();
    for movie in 0..movies_length {
        let sender = sender.clone();
        let button: Button = Button::builder()
            .label(MOVIES.lock().unwrap()[movie].name.clone())
            .build();
        button.connect_clicked(
            clone!(@weak info, @weak poster, @weak play_button => move |_| {
                let data: Option<MovieData> = MOVIES.lock().unwrap()[movie].data.clone();
                let poster_bytes: Option<glib::Bytes> = MOVIES.lock().unwrap()[movie].poster_bytes.clone();
                let sender = sender.clone();
                movie_selected(movie, poster_bytes, poster, play_button);
                match data {
                    Some(data) => {
                        show_info(&info, Some(data));
                    },
                    None => {
                        show_info(&info, None);
                        MOVIES.lock().unwrap()[movie].fetch_data();
                        if MOVIES.lock().unwrap()[movie].data.is_some() {
                            thread::spawn(move || {MOVIES.lock().unwrap()[movie].fetch_poster(movie, sender.clone());});
                            if let Some(data) = MOVIES.lock().unwrap()[movie].data.clone() {
                                show_info(&info, Some(data))
                            };
                        }
                    }
                }
            }),
        );
        list_box.add(&button);
    }
    reciever.attach(None, move |movie| {
        let poster_bytes = &MOVIES.lock().unwrap()[movie].poster_bytes;
        let bytes = match poster_bytes {
            Some(bytes) => glib::Bytes::from(bytes.clone()),
            None => glib::Bytes::from(LOADING_IMAGE_DARK),
        };
        let stream = MemoryInputStream::from_bytes(&bytes);
        let pixbuf = Pixbuf::from_stream(&stream, Cancellable::NONE).unwrap();
        let _ = &poster.set_pixbuf(Some(&pixbuf));
        // poster.set_can_shrink(true);
        poster.show();
        Continue(true)
    });

    main_window.show_all();
}

fn movie_selected(
    movie: usize,
    poster_bytes: Option<glib::Bytes>,
    poster: Image,
    play_button: Button,
) {
    let bytes = match poster_bytes {
        Some(bytes) => glib::Bytes::from(bytes.clone()),
        None => glib::Bytes::from(LOADING_IMAGE_DARK),
    };
    let stream = MemoryInputStream::from_bytes(&bytes);
    let pixbuf = Pixbuf::from_stream(&stream, Cancellable::NONE).unwrap();
    let _ = &poster.set_pixbuf(Some(&pixbuf));
    poster.show();
    *MOVIE_SELECTED.lock().unwrap() = movie;
    play_button.set_sensitive(true);
}

fn show_info(info: &Box, data: Option<MovieData>) {
    match data {
        Some(data) => {
            let text: [String; 8] = [
                format!("<b>Title:</b> {}", data.title),
                format!("<b>Original title:</b> {}", data.original_title),
                format!("<b>Original language:</b> {}", data.original_language),
                format!("<b>Overview:</b>\n {}", data.overview),
                format!("<b>Vote average (tmdb):</b> {}", data.vote_average),
                format!("<b>Vote count (tmdb):</b> {}", data.vote_count),
                format!("<b>Release date:</b> {}", data.release_date),
                format!("Play \"{}\"", data.title),
            ];
            let mut i: usize = 0;
            info.forall(|item| {
                item.set_property("label", &text[i]);
                i += 1;
            });
        }
        None => {
            let mut i: usize = 0;
            info.forall(|item| {
                item.set_property("label", &"".to_string());
                i += 1;
            });
        }
    }
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

fn user_dir(path: PathBuf) -> String {
    let mut path: PathBuf = path;
    path.push("movies");
    fs::create_dir_all(&path).expect("Couldn't create directory");
    path.to_str().unwrap().to_string()
}
