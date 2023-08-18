mod app;
mod detect;
mod main_window;
mod movie;
mod plugin;
mod res;
mod settings;
mod utils;

use gtk::glib::ExitCode;
use gtk::glib::{user_cache_dir, user_data_dir};
use std::{env, process::exit};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        let app = app::App::new();
        app.run()
    } else {
        match args[1].as_str() {
            "--version" | "-v" => {
                println!("Version: {}", env!("CARGO_PKG_VERSION"));
                exit(0)
            }
            "--help" | "-h" => {
                println!(
                    "Usage: {} [option]

  -v, --version \t show version and exit
  -h, --help \t\t show this help and exit
  -l, --list \t\t list all movies in library and exit
  -c, --clear-cache \t clear cache and exit (does not clear time positions)

You can add to library from within the app, or you can create symlinks to
directories with movies in \"{}/\"
(e.g. ln -s FULL_PATH_TO_DIRECTORY {}/NAME",
                    args[0],
                    utils::user_dir(user_data_dir()),
                    utils::user_dir(user_data_dir())
                );
                exit(0)
            }
            "--list" | "-l" => {
                let movies = detect::get_movies(utils::user_dir(user_data_dir()), vec![]);
                for movie in movies {
                    if let Some(year) = movie.year {
                        println!("{} ({})", movie.name, year);
                    } else {
                        println!("{}", movie.name);
                    }
                }
                exit(0)
            }
            "--clear-cache" | "-c" => {
                if let Result::Ok(_) = std::fs::remove_dir_all(utils::user_dir(user_cache_dir())) {
                    println!("Cache cleared");
                }
                exit(0)
            }
            _ => {
                println!("Unknown command line arguments");
                exit(1)
            }
        }
    }
}
