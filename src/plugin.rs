use std::io::{prelude::*, BufReader};
use std::path::PathBuf;
use std::process::{ChildStdin, Command, Stdio};

use glib::subclass::types::ObjectSubclassIsExt;
use glib::user_data_dir;
use reel_hub::movie::Movie;
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

pub fn handle_response(response: String, window: main_window::Window) {
    window.imp().movies.borrow_mut().push(
        Movie {
            name: "a".to_string(),
            year: None,
            file: PathBuf::from("/home/nikola/Media/0dcb85ee-4675-440a-ab94-14423585530f/movies/1408 (2007) [1080p]/1408.2007.1080p.BluRay.x264.YIFY.mp4"),
            data: None,
            current_time: None,
            duration: None,
            done: false,
        });
    window.update();
    println!("{}", response);
}
