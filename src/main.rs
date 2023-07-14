mod app;
mod main_window;

use glib::user_data_dir;
use gtk::glib::ExitCode;
use std::{env, process::exit};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        let app = app::App::new();
        app.run()
    } else if args.contains(&"--version".to_string()) || args.contains(&"-v".to_string()) {
        println!("Version: {}", env!("CARGO_PKG_VERSION"));
        exit(0)
    } else if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        println!("Usage: {} [option]

  -v, --version \t show version and exit
  -h, --help \t\t show this help and exit
  -l, --list \t\t list all movies in library and exit

You can add to library from within the app, or you can create symlinks to
directories with movies in \"{}/\"
(e.g. ln -s FULL_PATH_TO_DIRECTORY {}/NAME", args[0], reel_hub::utils::user_dir(user_data_dir()), reel_hub::utils::user_dir(user_data_dir()));
        exit(0)
    } else if args.contains(&"--list".to_string()) || args.contains(&"-l".to_string()) {
        let movies = reel_hub::detect::get_movies(reel_hub::utils::user_dir(user_data_dir()));
        for movie in movies {
            if let Some(year) = movie.year {
                println!("{} ({})", movie.name, year);
            } else {
                println!("{}", movie.name);
            }
        }
        exit(0)
    } else {
        println!("Unknown command line arguments");
        exit(1)
    }
}
