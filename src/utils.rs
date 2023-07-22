use std::{ffi::OsStr, fs::File};

use glib::user_cache_dir;

use crate::movie::{Movie, MovieCache};

pub fn user_dir(path: std::path::PathBuf) -> String {
    let mut path: std::path::PathBuf = path;
    path.push("reel_hub");
    std::fs::create_dir_all(&path).expect("Couldn't create directory");
    path.to_str().unwrap().to_string()
}

pub fn load_cache(movies: &mut Vec<Movie>) {
    let path = user_dir(user_cache_dir());
    let file = match File::open(format!("{}/{}", path, "movie_data.json")) {
        Ok(file) => file,
        Err(_) => return,
    };
    let cache: Result<Vec<MovieCache>, serde_json::Error> = serde_json::from_reader(file);
    if let Ok(cache) = cache {
        for movie in movies {
            match cache
                .iter()
                .find(|entry| OsStr::new(&entry.file_name) == movie.file.file_name().unwrap())
            {
                Some(entry) => {
                    movie.duration = Some(entry.duration);
                    movie.data = Some(entry.data.clone());
                }
                None => {}
            }
        }
    }
}
