mod app;
mod window;
mod res;

use gtk::glib::ExitCode;

// static LOADING_IMAGE_DARK: &[u8; 2904] = include_bytes!("pictures/Loading_dark.png");

fn main() -> ExitCode {
    let app = app::App::new();
    app.run()
}
