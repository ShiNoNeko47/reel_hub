use md5;
use std::{
    fs::File,
    io::{prelude::*, BufReader, Write},
    ops::Deref,
    path::PathBuf,
    process::{Child, Command, Stdio},
};

use gtk::glib::{user_cache_dir, user_data_dir};
use serde::{Deserialize, Serialize};

use crate::utils::{self, user_dir};

use self::tmdb::fetch_image_tmdb;

mod tmdb;

pub enum ImageType {
    Poster,
    Backdrop,
}

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
    pub backdrop_path: String,
    pub genres: Vec<String>,
}

#[derive(Debug)]
pub struct Movie {
    pub id: usize,
    pub name: String,
    pub year: Option<usize>,
    pub file: String,
    pub data: Option<MovieData>,
    pub current_time: Option<u32>,
    pub duration: Option<u32>,
    pub done: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MovieCache {
    pub file_name: String,
    pub duration: u32,
    pub done: bool,
    pub data: MovieData,
}

impl Movie {
    pub fn get_from_file_name(file: walkdir::DirEntry, id: usize) -> Movie {
        let re: regex::Regex = regex::Regex::new(
            r"(?mU)^(([A-Za-z0-9\. \(\)]+)[\. -]+\(?((\d{4})[^p]).*)|((.*)\.[A-Za-z0-9]+)$",
        )
        .unwrap();
        let mut prefix = "";
        if file.metadata().is_ok() {
            let size = file.metadata().unwrap().len();
            if size == 0 {
                prefix = "~ "
            }
        }
        let captures: regex::Captures = re.captures(&file.file_name().to_str().unwrap()).unwrap();
        let current_time = Self::get_current_time(file.path().to_str().unwrap().to_string());
        Movie {
            id,
            name: prefix.to_string()
                + &if let Some(name) = captures.get(2) {
                    name.as_str()
                } else {
                    captures.get(6).unwrap().as_str()
                }
                .replace(".", " "),
            year: if let Some(year) = captures.get(4) {
                Some(year.as_str().parse().unwrap())
            } else {
                None
            },
            file: file.path().to_owned().to_string_lossy().to_string(),
            data: None,
            current_time: if current_time == Some(0) {
                None
            } else {
                current_time
            },
            duration: None,
            done: false,
        }
    }

    pub fn get_current_time(file_path: String) -> Option<u32> {
        let mut current_time: Option<u32> = None;
        let hash = md5::compute::<String>(file_path);
        if let Ok(file) = File::open(
            utils::user_dir(user_data_dir())
                + "/.watch-later/"
                + &format!("{:x}", hash).to_uppercase(),
        ) {
            let mut reader = BufReader::new(file);
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            current_time = Some(
                line.trim()
                    .split("=")
                    .last()
                    .unwrap()
                    .parse::<f32>()
                    .unwrap_or(0.0) as u32,
            );
        }
        current_time
    }

    pub fn get_progress(&self) -> Option<u32> {
        let current_time = self.current_time.unwrap_or(0);
        let duration = self.duration;
        if duration == Some(0) || duration.is_none() {
            return duration;
        }
        if current_time == 0 {
            return Some(0);
        }
        Some((current_time as f32 / duration.unwrap() as f32 * 100.0) as u32 + 1)
    }

    pub fn play(&self, continue_watching: bool) -> Child {
        std::fs::create_dir_all(utils::user_dir(user_data_dir()) + "/.watch-later").unwrap();
        println!("Playing {}", self.name);
        Command::new("mpv")
            .arg(&self.file.deref())
            .arg("--no-config")
            .arg("--save-position-on-quit")
            .arg("--watch-later-options=start")
            .arg(format!(
                "--watch-later-directory={}/.watch-later",
                super::utils::user_dir(user_data_dir())
            ))
            .arg("--fs")
            .arg(if !continue_watching {
                "--no-resume-playback"
            } else {
                ""
            })
            .arg("--ytdl-format=mp4")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Movie failed to play")
    }

    pub fn fetch_image(
        image_path: String,
        image_type: ImageType,
        sender: gtk::glib::Sender<(PathBuf, ImageType)>,
    ) {
        let path = PathBuf::from(format!("{}{}", user_dir(user_cache_dir()), image_path));
        let mut file = match File::create(&path) {
            Ok(file) => file,
            Err(_) => {
                let _ = std::fs::create_dir_all(path.parent().unwrap());
                File::create(&path).unwrap()
            }
        };
        file.write(&fetch_image_tmdb(image_path).to_vec().to_vec())
            .expect("Couldn't write to file");
        sender.send((path, image_type)).expect("Couldn't send");
    }
}

impl PartialEq for Movie {
    fn eq(&self, other: &Self) -> bool {
        if PathBuf::from(&self.file).is_file() && PathBuf::from(&other.file).is_file() {
            return PathBuf::from(&self.file).file_name() == PathBuf::from(&other.file).file_name();
        }
        self.file == other.file
    }
}

impl Eq for Movie {}

impl PartialOrd for Movie {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Movie {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.current_time.is_some() {
            if other.current_time.is_some() {
                return self.name.cmp(&other.name);
            }
            return std::cmp::Ordering::Less;
        } else if other.current_time.is_some() {
            return std::cmp::Ordering::Greater;
        }

        if self.done == other.done {
            return self.name.cmp(&other.name);
        }
        if self.done {
            return std::cmp::Ordering::Greater;
        }
        std::cmp::Ordering::Less
    }
}

impl Clone for Movie {
    fn clone(&self) -> Self {
        Movie {
            id: self.id,
            name: self.name.clone(),
            year: self.year,
            file: self.file.clone(),
            data: self.data.clone(),
            current_time: self.current_time,
            duration: self.duration,
            done: self.done,
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
            backdrop_path: self.backdrop_path.clone(),
            genres: self.genres.clone(),
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

impl Clone for MovieCache {
    fn clone(&self) -> Self {
        MovieCache {
            file_name: self.file_name.clone(),
            duration: self.duration,
            done: self.done,
            data: self.data.clone(),
        }
    }
}
