use walkdir::WalkDir;

use crate::{movie::Movie, res};

pub fn get_movies(dir: String, movies: Vec<Movie>) -> Vec<Movie> {
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

    let movies_new: Vec<Movie> = files
        .iter()
        .map(|file| Movie::get_from_file_name(file.clone()))
        .collect();
    let mut movies = movies
        .into_iter()
        .filter(|movie| movies_new.contains(&movie))
        .collect::<Vec<Movie>>();

    for movie in movies_new {
        if !movies.contains(&movie) {
            movies.push(movie);
        }
    }
    movies
}
