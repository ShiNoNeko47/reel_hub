use std::fs::File;

use glib::user_data_dir;

use crate::movie::{Movie, MovieCache};

pub fn user_dir(path: std::path::PathBuf) -> String {
    let mut path: std::path::PathBuf = path;
    path.push("movies");
    std::fs::create_dir_all(&path).expect("Couldn't create directory");
    path.to_str().unwrap().to_string()
}

pub fn load_cache(movies: &mut Vec<Movie>) {
    let mut path = user_dir(user_data_dir());
    path.push_str("/cache");
    let file = File::open(path).unwrap();
    let cache: Vec<MovieCache> = serde_json::from_reader(file).unwrap();
    for movie in movies {
        match cache.iter().find(|entry| entry.file == movie.file) {
            Some(entry) => {
                movie.data = Some(entry.data.clone());
            }
            None => {}
        }
    }
}
