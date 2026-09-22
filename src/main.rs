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

    let app = adw::Application::builder()
        .application_id("com.github.glance.Glance")
        .flags(gio::ApplicationFlags::HANDLES_OPEN)
        .build();

    app.connect_activate(window::on_activate);
    app.connect_open(window::on_open);

    app.run();
}
