use gtk::prelude::*;
use gtk4 as gtk;

use crate::{desktop, launcher};

pub fn populate(app_container: &gtk::Box, target_url: Option<String>) {
    if target_url.is_none() {
        let no_url_label = gtk::Label::new(Some("No URL provided"));
        app_container.append(&no_url_label);
    }

    let dir_entries = desktop::installed_entries();
    // check if the linko directory is empty
    if dir_entries.is_empty() {
        let empty_label = gtk::Label::new(Some("There's no desktop files!"));
        app_container.append(&empty_label);
    } else {
        // for every app, create a button and append it to the container.
        for (app_path_string, app) in dir_entries {
            // UI things: one horizontal row per browser:
            // expanding browser button (icon + name) + small remove button.
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);

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
                .hexpand(true)
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
                match launcher::launch(&app, url) {
                    Ok(_) => std::process::exit(0),
                    Err(e) => {
                        eprintln!("Error: Failed to open URL with desired browser: {}", e);
                    }
                }
            });
            button_container.append(&image);
            button_container.append(&label);

            let remove_image = gtk::Image::from_icon_name("user-trash-symbolic");
            let remove_button = gtk::Button::builder()
                .child(&remove_image)
                .tooltip_text("Remove")
                .css_classes(vec!["destructive-action"])
                .build();

            let container_for_remove = app_container.clone();
            let url_for_remove = target_url.clone();
            let file_name_for_remove = app_path_string.clone();
            remove_button.connect_clicked(move |_| {
                if let Err(e) = desktop::remove_entry(&file_name_for_remove) {
                    eprintln!(
                        "Error: Failed to remove desktop entry {}: {}",
                        file_name_for_remove, e
                    );
                    return;
                }
                while let Some(child) = container_for_remove.first_child() {
                    container_for_remove.remove(&child);
                }
                populate(&container_for_remove, url_for_remove.clone());
            });

            row.append(&button);
            row.append(&remove_button);
            app_container.append(&row);
        }
    }
}
