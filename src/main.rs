use glib::{user_data_dir, MainContext, PRIORITY_DEFAULT};
use gtk::{
    glib::ExitCode, prelude::*, Box, Button, Label, ListBox, Orientation, Picture, ScrolledWindow,
    Widget, Window,
};
use libadwaita::Application;

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use movies::{
    detect,
    movie::{Movie, MovieData},
};

static MOVIES: Mutex<Vec<Movie>> = Mutex::new(vec![]);

static MOVIE_SELECTED: Mutex<usize> = Mutex::new(0);

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

    *MOVIES.lock().unwrap() = detect::get_movies(user_dir(user_data_dir()));

    if MOVIES.lock().unwrap().len() == 0 {
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
    for _ in 0..7 {
        info.append(
            &Label::builder()
                .use_markup(true)
                .wrap(true)
                .justify(gtk::Justification::Center)
                .build(),
        );
    }
    content.append(&info);

    let play_button = Button::builder().label("Play").build();
    play_button.connect_clicked(move |_| {
        MOVIES.lock().unwrap()[*MOVIE_SELECTED.lock().unwrap()].play(false)
    });
    play_button.set_sensitive(false);
    content.append(&play_button);

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

    let (movie_selected_sender, movie_selected_reciever) = MainContext::channel(PRIORITY_DEFAULT);
    let (info_loaded_sender, info_loaded_reciever) = MainContext::channel(PRIORITY_DEFAULT);
    let movies_length = MOVIES.lock().unwrap().len();
    for movie in 0..movies_length {
        let movie_selected_sender = movie_selected_sender.clone();
        let info_loaded_sender = info_loaded_sender.clone();
        let button = Button::builder()
            .label(MOVIES.lock().unwrap()[movie].name.clone())
            .build();
        button.connect_clicked(move |_| {
            movie_selected_sender
                .send(movie.clone())
                .expect("Couldn't send");
            let data = MOVIES.lock().unwrap()[movie].data.clone();
            match data {
                Some(data) => {
                    info_loaded_sender.send(data).expect("Couldn't send");
                }
                None => {
                    MOVIES.lock().unwrap()[movie].fetch_data();
                    if let Some(data) = MOVIES.lock().unwrap()[movie].data.clone() {
                        info_loaded_sender.send(data).expect("Couldn't send");
                    };
                }
            }
        });
        list_box.append(&button);
    }

    info_loaded_reciever.attach(None, move |movie_data| {
        show_info(&info, movie_data);
        Continue(true)
    });

    movie_selected_reciever.attach(None, move |movie| {
        *MOVIE_SELECTED.lock().unwrap() = movie;
        play_button.set_sensitive(true);
        Continue(true)
    });

    main_window.present();
}

fn show_info(info: &Box, data: MovieData) {
    let text = [
        format!("<b>Title:</b> {}", data.title),
        format!("<b>Original title:</b> {}", data.original_title),
        format!("<b>Original language:</b> {}", data.original_language),
        format!("<b>Overview:</b>\n {}", data.overview),
        format!("<b>Vote average (tmdb):</b> {}", data.vote_average),
        format!("<b>Vote count (tmdb):</b> {}", data.vote_count),
        format!("<b>Release date:</b> {}", data.release_date),
    ];
    let mut i = 0;
    info.observe_children().into_iter().for_each(|item| {
        item.unwrap().set_property("label", &text[i]);
        i += 1;
    });
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
    let mut path = path;
    path.push("movies");
    fs::create_dir_all(&path).expect("Couldn't create directory");
    path.to_str().unwrap().to_string()
}
