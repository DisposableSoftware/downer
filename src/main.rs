#[cfg(test)]
mod spec_tests;

use gtk::{glib, prelude::*};

pub const APP_ID: &str = "io.github.DisposableSoftware.Downer";

fn main() -> glib::ExitCode {
    // Without this, windows other than ApplicationWindows get the binary name as their
    // Wayland app_id, so compositor window rules miss dialogs.
    glib::set_prgname(Some(APP_ID));
    let app = gtk::Application::builder().application_id(APP_ID).build();
    app.connect_activate(|app| {
        gtk::ApplicationWindow::builder()
            .application(app)
            .title("Downer")
            .default_width(900)
            .default_height(700)
            .build()
            .present();
    });
    app.run()
}
