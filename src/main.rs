mod config;
mod editor;
mod outline;
mod sidebar;
mod viewer;
mod window;

use adw::prelude::*;
use gtk::gio;

fn main() {
    adw::init().expect("Failed to initialize libadwaita");

    if let Some(display) = gtk::gdk::Display::default() {
        let theme = gtk::IconTheme::for_display(&display);
        theme.add_search_path("data/icons");
        theme.add_search_path(
            &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/icons"),
        );
        theme.add_search_path("/usr/share/icons");
    }

    let app = adw::Application::builder()
        .application_id("com.github.glance.Glance")
        .flags(gio::ApplicationFlags::HANDLES_OPEN)
        .build();

    app.connect_activate(window::on_activate);
    app.connect_open(window::on_open);

    app.run();
}
