use crate::util;
use gtk::prelude::*;
use gtk4 as gtk;
use std::fs::read_dir;

pub fn entry(app_container: &gtk::Box, target_url: Option<String>) {
    if target_url.is_none() {
        let no_url_label = gtk::Label::new(Some("No URL provided"));
        app_container.append(&no_url_label);
    }

    let dir_entries = match read_dir(util::get_app_path()) {
        Ok(entries) => entries.collect::<Vec<_>>(),
        Err(e) => {
            eprintln!("Error: Failed to read application directory: {}", e);
            let error_label = gtk::Label::new(Some("Failed to read application directory"));
            app_container.append(&error_label);
            return;
        }
    };
    // check if the linko directory is empty
    if dir_entries.is_empty() {
        let empty_label = gtk::Label::new(Some("There's no desktop files!"));
        app_container.append(&empty_label);
    } else {
        // for every app, create a button and append it to the container.
        for app_desktop_path in dir_entries {
            let app_desktop_path = match app_desktop_path {
                Ok(entry) => entry,
                Err(e) => {
                    eprintln!("Error: Failed to open an app path: {}", e);
                    continue;
                }
            };
            // if the file is not a desktop entry, skips it.
            let app_path_string = app_desktop_path.file_name().to_string_lossy().into_owned();
            if !app_path_string.ends_with(".desktop") {
                println!("\n{} is not a desktop entry, skipping it.", app_path_string);
                continue;
            }

            let app = match util::parse_desktop_file(app_desktop_path) {
                Some(app) => app,
                None => {
                    eprintln!(
                        "Warning: Skipping unreadable desktop entry: {}",
                        app_path_string
                    );
                    continue;
                }
            };

            // UI things
            let button_container = gtk::Box::new(gtk::Orientation::Horizontal, 15);
            button_container.set_height_request(30);

            let label_class = vec!["body"];

            let label = gtk::Label::builder()
                .label(&app.name)
                .width_chars(5)
                .css_classes(label_class)
                .single_line_mode(true)
                .build();

            let image = gtk::Image::builder()
                .icon_name(&app.icon)
                .pixel_size(24)
                .build();

            let button_classes = vec!["button1"];
            let button = gtk::Button::builder()
                .css_classes(button_classes)
                .child(&button_container)
                .build();

            // when button is clicked, open the link
            // using the chosen browser
            let url_for_button = target_url.clone();
            button.connect_clicked(move |_| {
                let url = match url_for_button.as_deref() {
                    Some(url) => url,
                    None => {
                        eprintln!("Error: No URL to open");
                        return;
                    }
                };
                match util::launch(&app, url) {
                    Ok(_) => std::process::exit(0),
                    Err(e) => {
                        eprintln!("Error: Failed to open URL with desired browser: {}", e);
                    }
                }
            });
            button_container.append(&image);
            button_container.append(&label);
            app_container.append(&button);
        }
    }
}
