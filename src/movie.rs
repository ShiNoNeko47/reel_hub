use std::{process::{Command, Stdio}, path::PathBuf, ops::Deref, fs::File, io::Write};

use glib::{user_data_dir, user_cache_dir};
use serde::{Deserialize, Serialize};

use crate::utils::user_dir;

use self::tmdb::fetch_poster_tmdb;

mod tmdb;

#[derive(Debug, Serialize, Deserialize)]
pub struct MovieData {
    pub title: String,
    pub original_title: String,
    pub original_language: String,
    pub overview: String,
    pub vote_average: f64,
    pub vote_count: u64,
    pub release_date: String,
    pub poster_path: String,
}

#[derive(Debug)]
pub struct Movie {
    pub name: String,
    pub year: Option<usize>,
    pub file: PathBuf,
    pub data: Option<MovieData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MovieCache {
    pub file: PathBuf,
    pub data: MovieData,
}

impl Movie {
    pub fn get_from_file_name(file: walkdir::DirEntry) -> Movie {
        let re: regex::Regex =
            regex::Regex::new(r"^(.*)[\.| ]([0-9]{4})?\.[\.|A-Z]*[[0-9]+p]*.*mp4").unwrap();
        let size = file.metadata().unwrap().len();
        let mut prefix = "";
        if size == 0 {
            prefix = "~ "
        }
        let binding: Option<regex::Captures> = re.captures(&file.file_name().to_str().unwrap());
        match &binding {
            Some(expr) => Movie {
                name: prefix.to_string() + &expr[1].to_string().replace(".", " "),
                year: Some(expr[2].parse().unwrap()),
                file: file.path().to_owned(),
                data: None,
            },
            None => Movie {
                name: prefix.to_string() + &file.file_name().to_str().unwrap().replace(".mp4", ""),
                year: None,
                file: file.path().to_owned(),
                data: None,
            },
        }
    }

    pub fn play(&self, from_start: bool) {
        Command::new("mpv")
            .arg(&self.file.deref())
            .arg("--save-position-on-quit")
            .arg(format!("--watch-later-directory={}/watch-later", super::utils::user_dir(user_data_dir())))
            .arg("--watch-later-options-remove=fullscreen")
            .arg("--write-filename-in-watch-later-config")
            .arg(if from_start { "--start=0%" } else { "" })
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Movie failed to play");
        println!("Playing {}", self.name);
    }

    pub fn fetch_data(&mut self) {
        let year: String = match &self.year {
            Some(year) => format!("&year={}", year),
            None => "".to_string(),
        };

        self.data = tmdb::fetch_data_tmdb(&self.name, year);
    }

    pub fn fetch_poster(poster_path: String, sender: glib::Sender<PathBuf> ) {
        let path = PathBuf::from(format!("{}{}", user_dir(user_cache_dir()), poster_path));
        let mut file = File::create(&path).unwrap();
        file.write( &fetch_poster_tmdb(poster_path, Some(500)).to_vec().to_vec()).expect("Couldn't write to file");
        sender.send(path).expect("Couldn't send");
    }
}

impl PartialEq for Movie {
    fn eq(&self, other: &Self) -> bool {
        self.file == other.file
    }
}

impl Clone for Movie {
    fn clone(&self) -> Self {
        Movie {
            name: self.name.clone(),
            year: self.year,
            file: self.file.clone(),
            data: self.data.clone(),
        }
    }
}

impl Clone for MovieData {
    fn clone(&self) -> Self {
        MovieData {
            title: self.title.clone(),
            original_title: self.original_title.clone(),
            original_language: self.original_language.clone(),
            overview: self.overview.clone(),
            vote_average: self.vote_average,
            vote_count: self.vote_count,
            release_date: self.release_date.clone(),
            poster_path: self.poster_path.clone(),
        }
    }
}
