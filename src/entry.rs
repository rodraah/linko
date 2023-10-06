use crate::util;
use gtk::prelude::*;
use gtk4 as gtk;
use std::{fs::read_dir, process::{Command, Stdio}};

pub fn entry(app_container: &gtk::Box) {
    let mut dir: Vec<std::fs::DirEntry> = read_dir(util::get_app_path()).unwrap().filter_map(|entry| entry.ok()).collect();
    // check if the linko directory is empty
    if dir.is_empty() {
        let empty_label = gtk::Label::new(Some("There's no desktop files!"));
        app_container.append(&empty_label);
    } else {
        dir.sort_by_key(|dir| dir.path());
        // for every app, create a button and append it to the container.
        for app_desktop_path in dir {
            let app_desktop_path = app_desktop_path;
            // if the file is not a desktop entry, skips it.
            let app_path_string = app_desktop_path.file_name().into_string().unwrap();
            if !app_path_string.ends_with(".desktop") {
                println!("\n{} is not a desktop entry, skipping it.", app_path_string);
                continue;
            }

            let app = util::parse_desktop_file(app_desktop_path);
            let app_display_name = app.0;
            let app_display_icon = app.1;
            //let command = app.2;

            // UI things
            let button_container = gtk::Box::new(gtk::Orientation::Horizontal, 15);
            button_container.set_height_request(30);

            let label_class = vec!["body"];

            let label = gtk::Label::builder()
                .label(&app_display_name)
                .width_chars(5)
                .css_classes(label_class)
                .single_line_mode(true)
                .build();

            let image = gtk::Image::builder()
                .icon_name(&app_display_icon)
                .pixel_size(24)
                .build();

            let button_classes = vec!["button1"];
            let button = gtk::Button::builder()
                .css_classes(button_classes)
                .child(&button_container)
                .build();

            // when button is clicked, open the link
            // using the chosen browser
            button.connect_clicked(move |_| {
                let command = &app.2;
                println!("{}", command);
                Command::new("sh")
                    .arg("-c")
                    .arg(command)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .expect("Failed to open URL with desired browser");
                std::process::exit(0);
            });
            button_container.append(&image);
            button_container.append(&label);
            app_container.append(&button);
        }
    }
}
