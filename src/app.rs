use crate::main_window;
use glib::subclass::types::ObjectSubclassIsExt;
use gtk::{prelude::*, Application};

pub struct App {
    app: Application,
}

impl App {
    pub fn new() -> Self {
        let app = Application::builder().build();
        let new_app = Self { app };
        new_app.app.connect_activate(Self::on_activate);
        new_app
    }

    pub fn run(&self) -> glib::ExitCode {
        self.app.run()
    }

    fn on_activate(app: &Application) {
        let window = main_window::Window::new(app);
        app.connect_shutdown(move |_| window.imp().cache());
    }
}
