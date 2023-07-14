mod app;
mod main_window;

use gtk::glib::ExitCode;
use std::{env, process::exit};

// static LOADING_IMAGE_DARK: &[u8; 2904] = include_bytes!("pictures/Loading_dark.png");

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        let app = app::App::new();
        app.run()
    } else if args.contains(&"--version".to_string()) || args.contains(&"-v".to_string()) {
        println!("Version: {}", env!("CARGO_PKG_VERSION"));
        exit(0);
    } else if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        println!("Usage: {} [options]\n", args[0]);
        println!("Gtk movie library browser written in rust\n");
        println!("Options:");
        println!("  -v, --version \t show version and exit");
        println!("  -h, --help \t\t show this help and exit");
        exit(0);
    } else {
        println!("Unknown command line arguments");
        exit(1);
    }
}
