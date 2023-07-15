use std::ffi::OsStr;

use gdk_pixbuf::{self, traits::PixbufLoaderExt};

const IMG_LOADING: &[u8] = include_bytes!("res/img/Loading_dark.png");
const IMG_CONNECTION: &[u8] = include_bytes!("res/img/Check_connection.png");
const FILE_TYPES: [&'static str; 11] = [
    "mp4", "mkv", "avi", "mov", "flv", "wmv", "mpg", "mpeg", "3gp", "webm", "vob",
];

pub fn loading() -> gdk_pixbuf::Pixbuf {
    let pixbuf = gdk_pixbuf::PixbufLoader::new();
    pixbuf.write(IMG_LOADING).unwrap();
    pixbuf.close().unwrap();

    pixbuf.pixbuf().unwrap()
}

pub fn check_connection() -> gdk_pixbuf::Pixbuf {
    let pixbuf = gdk_pixbuf::PixbufLoader::new();
    pixbuf.write(IMG_CONNECTION).unwrap();
    pixbuf.close().unwrap();

    pixbuf.pixbuf().unwrap()
}

pub fn check_filetype(ext: Option<&OsStr>) -> bool {
    match ext {
        Some(ext) => FILE_TYPES.contains(&ext.to_str().unwrap()),
        None => false,
    }
}
