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

fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(include_str!("style.css"));

    gtk::StyleContext::add_provider_for_display(
        &gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn build_ui(application: &adw::Application) {
    let window = gtk::Window::new();
    window.set_application(Some(application));
    window.set_default_width(300);
    window.set_default_height(320);
    window.set_title(Some("Linko"));
    window.set_resizable(false);

    load_css();

    let header_bar = gtk::HeaderBar::new();
    header_bar.set_decoration_layout(Some("icon:close"));

    // container for the buttons.
    let app_container = gtk::Box::new(gtk::Orientation::Vertical, 10);
    app_container.set_margin_top(30);
    app_container.set_margin_bottom(30);
    app_container.set_margin_start(30);
    app_container.set_margin_end(30);

    let scrolled_window = gtk::ScrolledWindow::new();
    
    window.set_titlebar(Some(&header_bar));
    window.show();

    // browser entries.
    entry::entry(&app_container);

    scrolled_window.set_child(Some(&app_container));
    
    // Create a button to copy the link to clipboard
    let mut clipboard_classes = Vec::new();
    clipboard_classes.push("heading");
    clipboard_classes.push("pill");
    clipboard_classes.push("button1");

    let clipboard_button = gtk::Button::builder()
        .label("Copy to clipboard").css_classes(clipboard_classes)
        .build();

    // On click it copies the link to clipboard
    clipboard_button.connect_clicked(move |_| {
        let pre_link:Vec<String> = std::env::args().collect();
        let link = pre_link[1].clone();
        let display = gdk::Display::default().unwrap();
        let clipboard = display.clipboard();
        clipboard.set_text(&link);
    });
    
    app_container.append(
        &gtk::Separator::new(gtk::Orientation::Horizontal));
    app_container.append(&clipboard_button);
    window.set_child(Some(&scrolled_window));
}
