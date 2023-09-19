use gtk::gdk;
use gtk::prelude::*;
use gtk4 as gtk;

mod entry;
mod util;

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

    gtk::style_context_add_provider_for_display(
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
    window.set_icon_name(Some("linko"));
    //window.set_resizable(false);

    // Window box to append the entries and the clipboard button widgets
    let window_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    window_box.set_margin_top(30);
    window_box.set_margin_bottom(15);
    window_box.set_margin_start(30);
    window_box.set_margin_end(30);

    load_css();

    let header_bar = gtk::HeaderBar::new();
    header_bar.set_decoration_layout(Some("menu:close"));

    // Container for the entries.
    let app_container = gtk::Box::new(gtk::Orientation::Vertical, 10);

    // Scrolled window for the entries.
    let scrolled_window = gtk::ScrolledWindow::new();
    scrolled_window.set_vexpand(true);

    // Set headerbar and show the window to minimize startup time
    window.set_titlebar(Some(&header_bar));
    window.show();

    // Add entries button
    let add_button = gtk::Button::builder()
        .label("+")
        .css_classes(vec!["add_button"])
        .build();
    header_bar.pack_start(&add_button);

    add_button.connect_clicked(move |_| {
        util::add_entries_dialog();
        // TODO!!
        // util::reload_entries(&app_container);
    });

    // Browser entry parser
    entry::entry(&app_container);

    scrolled_window.set_child(Some(&app_container));

    // Create a button to copy the link to clipboard
    let clipboard_classes = vec!["heading", "pill", "button1"];

    let clipboard_label = "Copy to clipboard";
    let clipboard_button = std::rc::Rc::new(
        gtk::Button::builder()
            .css_classes(clipboard_classes)
            .margin_top(10)
            .build(),
    );
    clipboard_button.set_label(clipboard_label);

    // clone the button to change its value when clicked
    let clipboard_button_clone = clipboard_button.clone();
    clipboard_button.connect_clicked(move |_| {
        let pre_link: Vec<String> = std::env::args().collect();
        let link = pre_link[1].clone();
        let display = gdk::Display::default().unwrap();
        let clipboard = display.clipboard();
        clipboard.set_text(&link);
        clipboard_button_clone.set_label("Copied to clipboard!");
    });

    let action_bar = gtk::ActionBar::new();
    action_bar.set_center_widget(Some(clipboard_button.upcast_ref::<gtk::Button>()));
    window_box.append(&scrolled_window);
    window_box.append(&action_bar);
    window.set_child(Some(&window_box));
}
