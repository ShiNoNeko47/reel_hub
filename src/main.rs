use glib::{clone, user_data_dir, MainContext, PRIORITY_DEFAULT};
use gtk::{
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
    for movie in movies {
        let button = Button::builder().label(movie.name.clone()).build();
        let movie = movie.clone();
        let movie_selected_sender = movie_selected_sender.clone();
        button.connect_clicked(move |_| unsafe {
            MOVIE_SELECTED = Some(movie.clone());
            movie_selected_sender.send(()).expect("Couldn't send");
        });
        list_box.append(&button);
    }
    movie_selected_reciever.attach(None, move |_| {
        play_button.set_sensitive(true);
        Continue(true)
    });

    // for movie in movies {
    //     println!("{:#?}", movie);
    // }

    main_window.present();
}

fn show_info(info: &Box, data: MovieData) {
    info.append(
        &Label::builder()
            .label(&format!("<b>Original title:</b> {}", data.original_title))
            .use_markup(true)
            .build(),
    );
    info.append(
        &Label::builder()
            .label(&format!(
                "<b>Original language:</b> {}",
                data.original_language
            ))
            .use_markup(true)
            .build(),
    );
    info.append(
        &Label::builder()
            .label(&format!("<b>Overview:</b>\n {}", data.overview))
            .use_markup(true)
            .wrap(true)
            .justify(gtk::Justification::Center)
            .build(),
    );
    info.append(
        &Label::builder()
            .label(&format!(
                "<b>Vote average (tmdb):</b> {}",
                data.vote_average
            ))
            .use_markup(true)
            .build(),
    );
    info.append(
        &Label::builder()
            .label(&format!("<b>Vote count (tmdb):</b> {}", data.vote_count))
            .use_markup(true)
            .build(),
    );
    info.append(
        &Label::builder()
            .label(&format!("<b>Release date:</b> {}", data.release_date))
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

fn user_dir(path: PathBuf) -> String {
    let mut path = path;
    path.push("movies");
    fs::create_dir_all(&path).expect("Couldn't create directory");
    path.to_str().unwrap().to_string()
}
