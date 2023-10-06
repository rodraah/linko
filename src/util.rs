use std::{
    fs::{self, create_dir_all, DirEntry, File},
    io::Read,
    path::PathBuf,
};

use gtk::{gio, prelude::*};
use gtk4 as gtk;

pub fn split_string(line: &str, cut: &str, position: usize) -> String {
    let cut = line.split_once(cut).unwrap_or_default();
    // Isn't necessary to handle the case where the string isn't cutted
    match position {
        0 => cut.0.to_string(),
        1 => cut.1.to_string(),
        _ => panic!(),
    }
}

pub fn get_app_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| {
        println!("Error: Failed to read home directory");
        std::process::exit(1);
    });
    let apps_path = PathBuf::from(home).join(".config/linko");
    if !apps_path.exists() {
        create_dir_all(&apps_path).unwrap_or_else(|e| {
            println!("Error: Failed to create config directory: {}", e);
        });
    }
    apps_path
}

pub fn add_entries_dialog() -> gtk::FileChooserDialog {
    let dialog = gtk::FileChooserDialog::builder()
        .title("Choose your browser's desktop entry: ")
        .action(gtk::FileChooserAction::Open)
        .build();
    let default_path = gio::File::for_path("/usr/share/applications");
    dialog
        .set_current_folder(Some(&default_path))
        .unwrap_or_else(|e| {
            println!(
                "Error: Failed to read \"usr/share/applications\" folder: {}",
                e
            );
            std::process::exit(1)
        });
    dialog.add_buttons(&[
        ("Open", gtk::ResponseType::Accept),
        ("Cancel", gtk::ResponseType::Cancel),
    ]);

    // Filters desktop entries
    let filter = gtk::FileFilter::new();
    filter.add_mime_type("application/x-desktop");
    dialog.set_filter(&filter);

    dialog.connect_response(move |dialog, response| {
        if response == gtk::ResponseType::Accept {
            let file = dialog.file().unwrap();
            let file_name: String = file
                .path()
                .unwrap()
                .as_path()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned();
            let path = file.path().unwrap().as_path().to_owned();
            let destination = get_app_path().as_path().to_str().unwrap().to_owned();
            let destination = format!("{destination}/{file_name}");
            fs::copy(path, destination).unwrap();
        };
        dialog.hide()
    });
    dialog
}

pub fn parse_desktop_file(app_desktop_path: DirEntry) -> (String, String, String) {
    let app_desktop_path = app_desktop_path.path();
    let mut desktop_file =
        File::open(app_desktop_path).expect("Error: Failed to open the desktop file");
    let mut desktop_content = String::new();
    desktop_file
        .read_to_string(&mut desktop_content)
        .expect("Error: Failed to read the desktop file");
    // Set fallback values for app_display_name, app_exec_command and app_display_icon.
    let mut app_display_name = String::new();
    let mut app_exec_command = String::new();
    let mut app_display_icon: String = "web-browser".to_string();
    // Check if the exec command contains %u and %U
    let mut contains_u = false;
    let mut contains_upper_u = false;
    // Check if it's the first entry for name, exec and icon
    let mut name_check = false;
    let mut exec_check = false;
    let mut icon_check = false;
    for line in desktop_content.lines() {
        if line.starts_with('#') {
            continue;
        };
        if line.starts_with("Name=") && !name_check {
            app_display_name = split_string(line, "Name=", 1);
            name_check = true;
            continue;
        };
        if line.starts_with("Exec=") && !exec_check {
            app_exec_command = split_string(line, "Exec=", 1);
            if line.contains("%u") {
                contains_u = true;
            } else if line.contains("%U") {
                contains_upper_u = true;
            };
            exec_check = true;
            continue;
        };
        if line.starts_with("Icon=") && !icon_check {
            app_display_icon = split_string(line, "Icon=", 1);
            icon_check = true;
            continue;
        };
    }
    // Get the link from command-line arguments and check if
    // a link is provided
    let args_vec: Vec<String> = std::env::args().collect();
    if args_vec.len() < 2 {
        println!("Error: There's no link!");
        std::process::exit(1);
    }
    let link = &args_vec[1];

    // TODO! use string.replace instead of creating three variables
    let mut exec_command = app_exec_command;
    let mut post_command = String::new();
    if contains_u {
        post_command = split_string(&exec_command, " %u", 1);
        exec_command = split_string(&exec_command, " %u", 0);
    } else if contains_upper_u {
        post_command = split_string(&exec_command, " %U", 1);
        exec_command = split_string(&exec_command, " %U", 0);
    }
    let command = format!("{} {} {}", exec_command, link, post_command);
    (app_display_name, app_display_icon, command)
}
