use walkdir::WalkDir;

use crate::{movie::Movie, res};

pub fn get_movies(dir: String) -> Vec<Movie> {
    let walkdir = WalkDir::new(dir).follow_links(true);
    let files: Vec<walkdir::DirEntry> = walkdir
        .sort_by_key(|a| a.file_name().to_str().unwrap().to_owned())
        .into_iter()
        .filter_map(|file| file.ok())
        .filter_map(|file| {
            if res::check_filetype(file.path().extension()) {
                Some(file)
            } else {
                None
            }
        })
        .collect();
    let mut movies: Vec<Movie> = vec![];
    for file in files {
        movies.push(Movie::get_from_file_name(file))
    }
    movies
}
