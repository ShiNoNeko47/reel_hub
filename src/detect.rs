use walkdir::WalkDir;

use crate::{movie::Movie, res};

pub fn get_movies(dir: String, mut movies: Vec<Movie>) -> Vec<Movie> {
    let walkdir = WalkDir::new(dir).follow_links(true);
    let files: Vec<walkdir::DirEntry> = walkdir
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
    for file in files {
        let movie = Movie::get_from_file_name(file);
        if !movies.contains(&movie) {
            movies.push(movie);
        }
    }
    movies
}
