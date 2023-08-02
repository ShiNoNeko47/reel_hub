use std::io::{prelude::*, BufReader};
use std::process::{ChildStdin, Command, Stdio};

use glib::subclass::types::ObjectSubclassIsExt;
use glib::user_data_dir;
use reel_hub::movie::{Movie, MovieData};
use reel_hub::utils;

use crate::main_window;

pub fn load_plugins(sender: glib::Sender<String>) -> Vec<ChildStdin> {
    let mut path = utils::user_dir(user_data_dir());
    path.push_str("/.plugins/");
    std::fs::create_dir_all(&path).unwrap();

    let files: Vec<walkdir::DirEntry> = walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|file| file.ok())
        .filter(|file| file.file_type().is_file())
        .collect();
    let mut plugins = vec![];
    for file in files {
        let plugin = match Command::new(file.path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
        {
            Ok(plugin) => plugin,
            Err(e) => {
                println!("{}: Failed to spawn plugin: {}", file.path().display(), e);
                continue;
            }
        };
        let reader = BufReader::new(plugin.stdout.unwrap());
        plugins.push(plugin.stdin.unwrap());
        plugin_listen(reader, sender.clone());
    }
    plugins
}

fn plugin_listen(mut reader: BufReader<std::process::ChildStdout>, sender: glib::Sender<String>) {
    std::thread::spawn(move || loop {
        let mut buf = String::new();
        match reader.read_line(&mut buf) {
            Ok(_) => {
                if !buf.is_empty() {
                    sender.send(buf).unwrap()
                }
            }
            Err(e) => panic!("couldn't read stdout: {}", e),
        };
    });
}

pub fn handle_response(response: String, window: &main_window::Window) {
    let response = response.trim().split(";").collect::<Vec<&str>>();
    match response[0].to_lowercase().as_str() {
        "movie" => {
            if response.len() < 4 {
                return;
            }
            let movie = Movie {
                name: response[1].to_string(),
                year: response[2].parse::<usize>().ok(),
                file: response[3].to_string(),
                data: {
                    let mut data = response[4..].iter();
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
                },
                current_time: None,
                duration: None,
                done: false,
            };
            if window.imp().movies.borrow().contains(&movie) {
                return;
            }
            window.imp().movies.borrow_mut().push(movie);
            window.imp().movies.borrow_mut().sort_unstable();
            window
                .imp()
                .movies_len
                .replace(window.imp().movies.borrow().len());
            window.setup_buttons();
        }
        _ => {
            println!("{response:?}");
        }
    }
}
