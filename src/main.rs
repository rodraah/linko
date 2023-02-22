use gtk4 as gtk;
use gtk::prelude::*;
use gtk::gdk;

mod util;
mod entry;

fn main() {
    let application = adw::Application::builder()
        .application_id("rodraah.Linko")
        .build();
    application.connect_startup(build_ui);
    application.run();
}

fn build_ui(application: &adw::Application) {
    let window = gtk::Window::new();
    window.set_application(Some(application));
    window.set_default_width(300);
    window.set_title(Some("Linko"));
    window.set_resizable(false);

    let header_bar = gtk::HeaderBar::new();
    header_bar.set_decoration_layout(Some("icon:close"));

    // container for the buttons.
    let app_container = gtk::Box::new(gtk::Orientation::Vertical, 20);
    app_container.set_margin_top(30);
    app_container.set_margin_bottom(30);
    app_container.set_margin_start(30);
    app_container.set_margin_end(30);
    
    // browser entries.
    entry::entry(&app_container);
    
    // Create a button to copy the link to clipboard
    let clipboard_button = gtk::Button::with_label("Copy to clipboard");
    // On click it copies the link to clipboard
    clipboard_button.connect_clicked(move |_| {
        let pre_link:Vec<String> = std::env::args().collect();
        let link = pre_link[1].clone();
        let display = gdk::Display::default().unwrap();
        let clipboard = display.clipboard();
        clipboard.set_text(&link);
    });

    app_container.append(&clipboard_button);
    window.set_titlebar(Some(&header_bar));
    window.set_child(Some(&app_container));
    // Show the window.
    window.show();
}
