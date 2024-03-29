use crate::main_window;
use gtk::glib::subclass::types::ObjectSubclassIsExt;
use gtk::{gdk::Screen, prelude::*, Application, CssProvider, StyleContext};

pub struct App {
    app: Application,
}

impl App {
    pub fn new() -> Self {
        let app = Application::new(Some("com.github.ShiNoNeko47.reel_hub"), Default::default());
        let new_app = Self { app };
        new_app.app.connect_activate(Self::on_activate);
        new_app
    }

    pub fn run(&self) -> gtk::glib::ExitCode {
        self.app.run()
    }

    fn on_activate(app: &Application) {
        let css_provider = CssProvider::new();
        css_provider
            .load_from_data(include_bytes!("res/style/style.css").as_ref())
            .unwrap();
        StyleContext::add_provider_for_screen(
            &Screen::default().unwrap(),
            &css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        let window = main_window::Window::new(app);
        app.connect_shutdown(move |_| {
            window.imp().store_cache();
            window.imp().store_settings();
            for plugin in window.imp().plugins.borrow_mut().iter_mut() {
                let _ = plugin.write("0");
            }
        });
    }
}
