mod column;
mod config;
mod render;
mod sidebar;
#[cfg(test)]
mod spec_tests;
mod theme;
mod tree;
mod view;

use gtk::{gio, glib, prelude::*};

pub const APP_ID: &str = "io.github.DisposableSoftware.Downer";

fn main() -> glib::ExitCode {
    // Without this, windows other than ApplicationWindows get the binary name as their
    // Wayland app_id, so compositor window rules miss dialogs.
    glib::set_prgname(Some(APP_ID));
    let app = gtk::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::HANDLES_OPEN)
        .build();
    app.connect_startup(|_| theme::load(&config::Config::load()));
    app.connect_activate(view::open_empty);
    app.connect_open(|app, files, _| {
        for file in files {
            view::open(app, file);
        }
    });
    app.run()
}
