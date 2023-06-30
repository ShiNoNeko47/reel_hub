use gdk_pixbuf::{self, traits::PixbufLoaderExt};

const IMG_LOADING: &[u8] = include_bytes!("res/img/Loading_dark.png");

pub fn loading() -> gdk_pixbuf::Pixbuf {
    let pixbuf = gdk_pixbuf::PixbufLoader::new();
    pixbuf.write(IMG_LOADING).unwrap();
    pixbuf.close().unwrap();

    pixbuf.pixbuf().unwrap()
}
