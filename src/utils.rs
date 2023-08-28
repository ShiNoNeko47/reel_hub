use std::{ffi::OsStr, fs::File, path::PathBuf};

use gtk::glib::{user_cache_dir, user_config_dir};

use crate::{
    movie::{Movie, MovieCache},
    settings::Settings,
};

pub fn user_dir(path: std::path::PathBuf) -> String {
    let mut path: std::path::PathBuf = path;
    path.push("reel_hub");
    std::fs::create_dir_all(&path).expect("Couldn't create directory");
    path.to_str().unwrap().to_string()
}

pub fn load_cache(movies: &mut Vec<Movie>) -> Vec<MovieCache> {
    let path = user_dir(user_cache_dir());
    let file = match File::open(format!("{}/{}", path, "movie_data.json")) {
        Ok(file) => file,
        Err(_) => return vec![],
    };
    let cache: Result<Vec<MovieCache>, serde_json::Error> = serde_json::from_reader(file);
    if let Ok(cache) = cache {
        for movie in movies
            .iter_mut()
            .filter(|movie| movie.duration.is_none() || movie.data.is_none())
        {
            match cache.iter().find(|entry| {
                OsStr::new(&entry.file_name) == PathBuf::from(&movie.file).file_name().unwrap()
            }) {
                Some(entry) => {
                    movie.duration = Some(entry.duration);
                    movie.done = entry.done;
                    movie.data = Some(entry.data.clone());
                }
                None => {}
            }
        }
        cache
    } else {
        vec![]
    }
}

pub fn load_settings() -> Option<Settings> {
    let path = user_dir(user_config_dir());
    let file = match File::open(format!("{}/{}", path, "settings.json")) {
        Ok(file) => file,
        Err(_) => return None,
    };
    serde_json::from_reader(file).ok()
}
