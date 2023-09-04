use std::io::{prelude::*, BufReader};
use std::process::{ChildStdin, Command, Stdio};

use crate::movie::{Movie, MovieData};
use crate::utils;
use gtk::glib::subclass::types::ObjectSubclassIsExt;
use gtk::glib::user_data_dir;
use gtk::glib::{self, clone};
use gtk::prelude::*;
use gtk::subclass::window::WindowImpl;
use gtk::CssProvider;

use crate::main_window::{self, UserInputType};

pub fn load_plugins(
    sender: gtk::glib::Sender<(String, usize)>,
    skip: Vec<std::path::PathBuf>,
) -> Vec<(ChildStdin, walkdir::DirEntry, bool)> {
    let mut path = utils::user_dir(user_data_dir());
    path.push_str("/.plugins/");
    std::fs::create_dir_all(&path).unwrap();

    let files: Vec<walkdir::DirEntry> = walkdir::WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|file| file.ok())
        .filter(|file| file.file_type().is_file())
        .filter(|file| !skip.contains(&std::path::PathBuf::from(file.path())))
        .collect();
    let mut plugins = vec![];
    for file in files {
        let plugin = match Command::new(file.path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
        {
            Ok(plugin) => {
                println!("Loading {}...", file.file_name().to_string_lossy());
                plugin
            }
            Err(_) => {
                continue;
            }
        };
        let reader = BufReader::new(plugin.stdout.unwrap());
        plugins.push((plugin.stdin.unwrap(), file, true));
        plugin_listen(reader, sender.clone(), plugins.len() - 1);
    }
    println!("Loaded {} plugins", plugins.len());
    plugins
}

fn plugin_listen(
    mut reader: BufReader<std::process::ChildStdout>,
    sender: gtk::glib::Sender<(String, usize)>,
    plugin_id: usize,
) {
    std::thread::spawn(move || loop {
        let mut buf = String::new();
        match reader.read_line(&mut buf) {
            Ok(_) => {
                if !buf.is_empty() {
                    sender.send((buf, plugin_id)).unwrap()
                }
            }
            Err(e) => panic!("couldn't read stdout: {}", e),
        };
    });
}

pub fn handle_response(response: String, window: &main_window::Window, plugin_id: usize) {
    let response = response.trim().split(";").collect::<Vec<&str>>();
    match response[0].to_lowercase().as_str() {
        "get_images" => {
            let _ = window.imp().plugins.borrow_mut()[plugin_id].0.write_all(
                format!(
                    "poster;{}\n",
                    window.imp().poster.file().unwrap_or("".into()).to_string()
                )
                .as_bytes(),
            );
            if window.imp().backdrop.is_visible() {
                let _ = window.imp().plugins.borrow_mut()[plugin_id].0.write_all(
                    format!(
                        "backdrop;{}\n",
                        window
                            .imp()
                            .backdrop
                            .file()
                            .unwrap_or("".into())
                            .to_string()
                    )
                    .as_bytes(),
                );
            }
        }
        "get_user_input" => {
            let mut response = response[1..].iter();
            let (sender, receiver) = gtk::glib::MainContext::channel(gtk::glib::PRIORITY_DEFAULT);
            window.get_user_input(
                response.next().copied(),
                sender,
                match response.next().unwrap_or(&&"").to_lowercase().as_str() {
                    "password" => UserInputType::Password,
                    "choice" => UserInputType::Choice,
                    _ => UserInputType::Text,
                },
                response.map(|s| s.to_string()).collect(),
            );
            receiver.attach(None,
                clone!(@weak window => @default-return Continue(false), move |user_input| {
                    let _ = window.imp().plugins.borrow_mut()[plugin_id].0.write_all(format!("user_input;{}\n", user_input).as_bytes());
                    Continue(false)
                }),
            );
        }
        "update" => {
            window.update();
        }
        "movie" => {
            let id_prefix = plugin_id * 10000;
            let movie_id = window
                .imp()
                .movies
                .borrow()
                .iter()
                .map(|movie| {
                    if movie.id >= id_prefix && movie.id <= id_prefix + 10000 {
                        movie.id
                    } else {
                        id_prefix
                    }
                })
                .max()
                .map(|id| id + 1)
                .unwrap_or(id_prefix);
            let _ = window.imp().plugins.borrow_mut()[plugin_id]
                .0
                .write_all(format!("movie_id;{movie_id}\n").as_bytes());
            if response.len() < 4 {
                return;
            }
            let mut data = response[1..].iter();
            let movie = Movie {
                id: movie_id,
                name: data.next().unwrap().to_string(),
                year: data.next().unwrap().parse::<usize>().ok(),
                file: data.next().unwrap().to_string(),
                current_time: if let Some(time) = data.next() {
                    if time.is_empty() {
                        Movie::get_current_time(response[3].to_string())
                    } else {
                        Some(time.parse::<u32>().unwrap_or(0))
                    }
                } else {
                    Movie::get_current_time(response[2].to_string())
                },
                duration: data
                    .next()
                    .map(|duration| duration.parse::<u32>().unwrap_or(0))
                    .unwrap_or(0),
                done: data
                    .next()
                    .map(|done| done.parse::<bool>().unwrap_or(false))
                    .unwrap_or(false),
                data: {
                    if response.len() <= 7 {
                        None
                    } else {
                        Some(MovieData {
                            title: data.next().unwrap_or(&response[1]).to_string(),
                            original_title: data.next().unwrap_or(&"").to_string(),
                            overview: data.next().unwrap_or(&"").to_string(),
                            original_language: data.next().unwrap_or(&"").to_string(),
                            poster_path: data.next().unwrap_or(&"").to_string(),
                            backdrop_path: data.next().unwrap_or(&"").to_string(),
                            vote_average: data.next().unwrap_or(&"").parse::<f64>().unwrap_or(0.0),
                            vote_count: data.next().unwrap_or(&"").parse::<u64>().unwrap_or(0),
                            release_date: data.next().unwrap_or(&"").to_string(),
                            genres: data.map(|s| s.to_string()).collect::<Vec<String>>(),
                        })
                    }
                },
            };
            if window.imp().movies.borrow().contains(&movie) {
                return;
            }
            let movie_selected_id = match window.imp().movie_selected.get() {
                Some(idx) => Some(window.imp().movies.borrow()[idx].id),
                None => window.imp().movie_selected_tmp.get(),
            };
            window.imp().movies.borrow_mut().push(movie);
            window
                .imp()
                .cache
                .replace(utils::load_cache(&mut window.imp().movies.borrow_mut()));
            window.imp().movies.borrow_mut().sort_unstable();
            window
                .imp()
                .movies_len
                .replace(window.imp().movies.borrow().len());
            window.setup_buttons();
            if let Some(id) = movie_selected_id {
                let movie = window.imp().movies.borrow().iter().position(|x| x.id == id);
                window.imp().movie_select(movie);
                if let Some(movie) = movie {
                    window.imp().button_selected.set(movie);
                    window.imp().buttons.borrow()[movie].grab_focus();
                }
            }
        }
        "css" => {
            let provider = CssProvider::new();
            provider
                .load_from_data(response[1..].join("; ").as_bytes())
                .unwrap();
            gtk::StyleContext::add_provider_for_screen(
                &gtk::gdk::Screen::default().unwrap(),
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
        "move" => {
            let amount = response[1].parse::<i32>().unwrap_or(0);
            let button_selected = window.imp().button_selected.get() as i32 + amount;
            window
                .imp()
                .button_selected
                .replace(button_selected as usize);
            window.imp().buttons.borrow()[button_selected as usize].grab_focus();
        }
        "select" => {
            window.imp().activate_focus();
        }
        "select_id" => {
            if let Some(position) = window
                .imp()
                .movies
                .borrow()
                .iter()
                .position(|movie| movie.id == response[1].parse::<usize>().unwrap_or(0))
            {
                window.imp().button_selected.set(position);
                window.imp().buttons.borrow()[position].grab_focus();
                window.imp().activate_focus();
            }
        }
        "play" => {
            if window.imp().play_button.is_visible() {
                window.imp().play_button.activate();
            }
        }
        _ => {
            println!("{plugin_id:?} - {response:?}");
        }
    }
}
