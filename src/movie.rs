use std::{process::{Command, Stdio, Child}, path::PathBuf, ops::Deref, fs::File, io::{prelude::*, Write, BufReader}};
use md5;

use glib::{user_data_dir, user_cache_dir};
use serde::{Deserialize, Serialize};

use crate::utils::{user_dir, self};

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
    pub current_time: Option<u32>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MovieCache {
    pub file_name: String,
    pub data: MovieData,
}

impl Movie {
    pub fn get_from_file_name(file: walkdir::DirEntry) -> Movie {
        let re: regex::Regex =
            regex::Regex::new(r"(?mU)^(([A-Za-z0-9\. \(\)]+)[\. -]+\(?((\d{4})[^p]).*)|((.*)\.[A-Za-z0-9]+)$").unwrap();
        let mut prefix = "";
        if file.metadata().is_ok() {
            let size = file.metadata().unwrap().len();
            if size == 0 {
                prefix = "~ "
            }
        }
        let captures: regex::Captures = re.captures(&file.file_name().to_str().unwrap()).unwrap();
        let mut current_time: Option<u32> = None;
        let hash = md5::compute::<String>(file.path().to_str().unwrap().to_string());
        if let Ok(file) = File::open(utils::user_dir(user_data_dir()) + "/.watch-later/" + &format!("{:x}", hash).to_uppercase()) {
            let mut reader = BufReader::new(file);
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            current_time = Some(line.trim().split("=").last().unwrap().parse::<f32>().unwrap() as u32);
        }
        Movie {
            name: prefix.to_string() + &if let Some(name) = captures.get(2) {name.as_str()} else {captures.get(6).unwrap().as_str()}.replace(".", " "),
            year: if let Some(year) = captures.get(4) { Some(year.as_str().parse().unwrap()) } else { None },
            file: file.path().to_owned(),
            data: None,
            current_time: if current_time == Some(0) { None } else { current_time }
        }
    }

    pub fn play(&self, from_start: bool) -> Child {
        print!("Playing {}", self.name);
        Command::new("mpv")
            .arg(&self.file.deref())
            .arg("--no-config")
            .arg("--save-position-on-quit")
            .arg("--watch-later-options-remove=fullscreen")
            .arg(format!("--watch-later-directory={}/.watch-later", super::utils::user_dir(user_data_dir())))
            .arg("--fs")
            .arg(if from_start { "--start=0%" } else { "" })
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Movie failed to play")
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
        self.file.file_name() == other.file.file_name()
    }
}

impl Clone for Movie {
    fn clone(&self) -> Self {
        Movie {
            name: self.name.clone(),
            year: self.year,
            file: self.file.clone(),
            data: self.data.clone(),
            current_time: self.current_time
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

impl MovieData {
    pub fn fetch_data(year: Option<usize>, name: String) -> Option<MovieData> {
        let year: String = match year {
            Some(year) => format!("&year={}", year),
            None => "".to_string(),
        };

        let data = tmdb::fetch_data_tmdb(&name, year);
        data
    }
}
