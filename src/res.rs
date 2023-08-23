use std::ffi::OsStr;

use gtk::gdk_pixbuf::{self, traits::PixbufLoaderExt};

const IMG_LOADING_500: &[u8] = include_bytes!("res/img/Loading_500.png");
const IMG_CONNECTION_500: &[u8] = include_bytes!("res/img/Check_connection_500.png");
const IMG_LOADING_780: &[u8] = include_bytes!("res/img/Loading_780.png");
const IMG_CONNECTION_780: &[u8] = include_bytes!("res/img/Check_connection_780.png");
const FILE_TYPES: [&'static str; 11] = [
    "mp4", "mkv", "avi", "mov", "flv", "wmv", "mpg", "mpeg", "3gp", "webm", "vob",
];

pub const TMDB_GENRES: [(usize, &'static str); 19] = [
    (28, "Action"),
    (12, "Adventure"),
    (16, "Animation"),
    (35, "Comedy"),
    (80, "Crime"),
    (99, "Documentary"),
    (18, "Drama"),
    (10751, "Family"),
    (14, "Fantasy"),
    (36, "History"),
    (27, "Horror"),
    (10402, "Music"),
    (9648, "Mystery"),
    (10749, "Romance"),
    (878, "Science Fiction"),
    (10770, "TV Movie"),
    (53, "Thriller"),
    (10752, "War"),
    (37, "Western"),
];

pub fn loading(image_type: &super::movie::ImageType) -> gdk_pixbuf::Pixbuf {
    let image = match image_type {
        super::movie::ImageType::Poster => IMG_LOADING_500,
        super::movie::ImageType::Backdrop => IMG_LOADING_780,
    };
    let pixbuf = gdk_pixbuf::PixbufLoader::new();
    pixbuf.write(image).unwrap();
    pixbuf.close().unwrap();

    pixbuf.pixbuf().unwrap()
}

pub fn check_connection(image_type: &super::movie::ImageType) -> gdk_pixbuf::Pixbuf {
    let image = match image_type {
        super::movie::ImageType::Poster => IMG_CONNECTION_500,
        super::movie::ImageType::Backdrop => IMG_CONNECTION_780,
    };
    let pixbuf = gdk_pixbuf::PixbufLoader::new();
    pixbuf.write(image).unwrap();
    pixbuf.close().unwrap();

    pixbuf.pixbuf().unwrap()
}

pub fn check_filetype(ext: Option<&OsStr>) -> bool {
    match ext {
        Some(ext) => FILE_TYPES.contains(&ext.to_str().unwrap()),
        None => false,
    }
}
