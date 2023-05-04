use glib::{clone, user_data_dir, MainContext, PRIORITY_DEFAULT};
use gtk::{
    ffi::GtkLabel,
    gdk_pixbuf::Pixbuf,
    gio::{Cancellable, MemoryInputStream},
    glib::ExitCode,
    prelude::*,
    Box, Button, Label, ListBox, Orientation, Picture, ScrolledWindow, Widget, Window,
};
use libadwaita::Application;

use std::io::Read;
use std::path::PathBuf;
use std::{cell::RefCell, fs};

use movies::{
    detect,
    movie::{Movie, MovieData},
};

thread_local! {
    static MOVIES: RefCell<Vec<Movie>> = vec![].into();
}

static mut MOVIE_SELECTED: Option<Movie> = None;

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

    let movies = detect::get_movies(user_dir(user_data_dir()));

    if movies.len() == 0 {
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
    for _ in 0..6 {
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
    play_button.connect_clicked(move |_| unsafe { MOVIE_SELECTED.clone().unwrap().play(false) });
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
    for mut movie in movies {
        movie.fetch_data();
        let movie_selected_sender = movie_selected_sender.clone();
        let button = Button::builder().label(movie.name.clone()).build();
        button.connect_clicked(clone!(@weak info => move |_| {
            let movie = movie.clone();
            show_info(&info, movie.data.clone().unwrap());
            movie_selected_sender.send(movie).expect("Couldn't send");
        }));
        list_box.append(&button);
    }

    movie_selected_reciever.attach(None, move |movie| {
        unsafe {
            MOVIE_SELECTED = Some(movie);
        }
        play_button.set_sensitive(true);
        Continue(true)
    });

    main_window.present();
}

fn show_info(info: &Box, data: MovieData) {
    let text = [
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
