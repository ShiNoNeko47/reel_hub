use std::{fs::File, ffi::OsStr};

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
    let cache: Vec<MovieCache> = serde_json::from_reader(file).unwrap();
    for movie in movies {
        match cache.iter().find(|entry| OsStr::new(&entry.file_name) == movie.file.file_name().unwrap()) {
            Some(entry) => {
                movie.data = Some(entry.data.clone());
                let poster_path = format!("{}{}", path, entry.data.poster_path);
                match File::open(&poster_path) {
                    Ok(poster_path) => poster_path,
                    Err(_) => continue,
                };
                // movie.poster_file = Some(PathBuf::from(poster_path));
            }
            None => {}
        }
    }
}
