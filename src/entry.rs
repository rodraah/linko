use gtk4 as gtk;
use gtk::prelude::*;
use std::{
    fs::read_dir,
    process::Command
};
use crate::util;

pub fn entry(app_container: &gtk::Box) {    
    // check if the linko directory is empty
    let is_empty = read_dir(util::get_app_path()).unwrap().next().is_none();
    if is_empty {
        let empty_label = gtk::Label::new(Some("There's no desktop files!"));
        app_container.append(&empty_label);
    } else {
        // for every app, create a button and append it to the container.
        for app_desktop_path in read_dir(util::get_app_path()).unwrap() {
            let app_desktop_path = app_desktop_path.expect("Failed to open the app path");
            let app = util::parse_desktop_file(app_desktop_path);
            let app_display_name = app.0;
            let app_display_icon = app.1;
            let command = app.2;


            // UI things
            let button_container = gtk::Box::new(gtk::Orientation::Horizontal, 6);
            
            let label = gtk::Label::builder()
                .label(&app_display_name)
                .width_chars(5)
                .single_line_mode(true)
                .build();

            let button = gtk::Button::builder()
                .height_request(20)
                .width_request(20)
                .name(&app_display_name)
                .icon_name(&app_display_icon)
                .build();

            // when button is clicked, open the link 
            // using the chosen browser
            button.connect_clicked(move |_| {
                println!("{}", command);
                Command::new("sh")
                    .arg("-c")
                    .arg(command.clone())
                    .spawn()
                    .expect("Failed to open URL with desired browser");
                quit::with_code(1);
            });
            button_container.append(&button);
            button_container.append(&label);
            app_container.append(&button_container);
        }
    }
}
