use gtk::gdk;
use gtk::gio;
use gtk::prelude::*;
use gtk4 as gtk;
use std::cell::RefCell;
use std::rc::Rc;

mod desktop_entry;
mod entry;
mod util;

fn main() {
    let pending_url: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(util::get_target_url()));

    let application = adw::Application::builder()
        .application_id("rodraah.Linko")
        .flags(gio::ApplicationFlags::HANDLES_COMMAND_LINE)
        .build();

    application.connect_startup(|_| load_css());

    {
        let pending_url = pending_url.clone();
        application.connect_activate(move |application| {
            build_ui(application, pending_url.borrow().clone());
        });
    }

    {
        let pending_url = pending_url.clone();
        application.connect_command_line(move |application, command_line| {
            let args = command_line.arguments();
            let joined = args
                .iter()
                .skip(1)
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join(" ");
            let url = if joined.is_empty() {
                None
            } else {
                Some(joined)
            };
            *pending_url.borrow_mut() = url;
            application.activate();
            0
        });
    }

    application.run();
}

fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(include_str!("style.css"));

    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    } else {
        eprintln!("Error: Could not connect to a display.");
    }
}

fn build_ui(application: &adw::Application, target_url: Option<String>) {
    if let Some(window) = application.active_window() {
        window.present();
        return;
    }

    let window = gtk::ApplicationWindow::new(application);
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

    let header_bar = gtk::HeaderBar::new();
    header_bar.set_decoration_layout(Some("menu:close"));

    // Container for the entries.
    let app_container = gtk::Box::new(gtk::Orientation::Vertical, 10);

    // Scrolled window for the entries.
    let scrolled_window = gtk::ScrolledWindow::new();
    scrolled_window.set_vexpand(true);

    // Set headerbar and show the window to minimize startup time
    window.set_titlebar(Some(&header_bar));
    window.present();

    // Add entries button
    let add_button = gtk::Button::builder()
        .label("+")
        .css_classes(vec!["add_button"])
        .build();
    header_bar.pack_start(&add_button);

    add_button.connect_clicked({
        let parent = window.clone();
        let container = app_container.clone();
        let url = target_url.clone();
        move |_| {
            util::add_entries_dialog(&parent, &container, url.clone());
        }
    });

    // Browser entry parser
    entry::entry(&app_container, target_url.clone());

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
    if let Some(link) = target_url {
        clipboard_button.connect_clicked(move |_| {
            if let Some(display) = gdk::Display::default() {
                let clipboard = display.clipboard();
                clipboard.set_text(&link);
                clipboard_button_clone.set_label("Copied to clipboard!");
            } else {
                eprintln!("Error: Could not connect to a display.");
            }
        });
    } else {
        clipboard_button.set_label("No link to copy");
        clipboard_button.set_sensitive(false);
    }

    let action_bar = gtk::ActionBar::new();
    action_bar.set_center_widget(Some(clipboard_button.upcast_ref::<gtk::Button>()));
    window_box.append(&scrolled_window);
    window_box.append(&action_bar);
    window.set_child(Some(&window_box));
}
