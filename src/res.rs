use std::ffi::OsStr;

use gdk_pixbuf::{self, traits::PixbufLoaderExt};

const IMG_LOADING_500: &[u8] = include_bytes!("res/img/Loading_500.png");
const IMG_CONNECTION_500: &[u8] = include_bytes!("res/img/Check_connection_500.png");
const IMG_LOADING_780: &[u8] = include_bytes!("res/img/Loading_780.png");
const IMG_CONNECTION_780: &[u8] = include_bytes!("res/img/Check_connection_780.png");
const FILE_TYPES: [&'static str; 11] = [
    "mp4", "mkv", "avi", "mov", "flv", "wmv", "mpg", "mpeg", "3gp", "webm", "vob",
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
