use gtk4 as gtk;
use gtk::prelude::*;
use gtk::gdk;
use std::{
    fs::{create_dir_all, read_dir, File}, 
    path::PathBuf,
    io::Read,
    process::Command
};

fn main() {
    let application = gtk::Application::builder()
        .application_id("rodraah.Linko")
        .build();
    application.connect_startup(build_ui);
    application.run();
}

fn build_ui(application: &gtk::Application) {
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
    app_container.set_halign(gtk::Align::Start);
    app_container.set_valign(gtk::Align::Start);

    // check if the linko directory is empty
    let is_empty = read_dir(get_app_path().clone()).unwrap().next().is_none();
    // for every app, create a button and append it to the container.
    if is_empty == false {
        for app_desktop_path in read_dir(get_app_path().clone()).unwrap() {
            let app_desktop_path = app_desktop_path.expect("failed to open the app path").path();
            let mut desktop_file = File::open(app_desktop_path).expect("Failed to open the desktop file");
            let mut desktop_content = String::new();
            desktop_file.read_to_string(&mut desktop_content)
                .expect("Failed to read the desktop file");
            //TODO! Create functions to get the app_display_name, the app_exec_command and the app_display_icon from the desktop_content
            // Set fallback values for app_display_name, app_exec_command and app_display_icon.
            let mut app_display_name = String::new();
            let mut app_exec_command = String::new();
            let mut app_display_icon:String = "web-browser".to_string();
            // Check if the exec command contains %u and %U
            let mut contains_u = false;
            let mut contains_upper_u = false;
            // Check if it's the first entry for name, exec and icon
            let mut name_check = false;
            let mut exec_check = false;
            let mut icon_check = false;
            for line in desktop_content.lines() {
                if line.starts_with("#") {
                    continue;
                };
                if line.starts_with("Name=") == true {
                    if name_check == false {
                        app_display_name = split_string(line, "Name=", 1);
                        name_check = true;
                        continue;
                    };
                };
                if line.starts_with("Exec=") == true {
                    if exec_check == false {
                        app_exec_command = split_string(line, "Exec=", 1);
                        if line.contains("%u") == true {
                            contains_u = true;
                        } if line.contains ("%U") == true {
                            contains_upper_u = true;
                        };
                        exec_check = true;
                        continue;
                    };
                };
                if line.starts_with("Icon=") == true {
                    if icon_check == false {
                        app_display_icon = split_string(line, "Icon=", 1);
                        icon_check = true;
                        continue;
                    };
                };
            };
            // Get the link from command-line arguments and check if
            // a link is provided
            let args_vec:Vec<String> = std::env::args().collect();
            if args_vec.len() < 2 {
                println!("Error: There's no link!");
                quit::with_code(1);
            }
            let link = &args_vec[1];

            // TODO! use string.replace instead of creating three variables
            let mut exec_command = app_exec_command;
            let mut post_command = String::new();
            if contains_u == true {
                post_command = split_string(&exec_command, " %u", 1);
                exec_command = split_string(&exec_command, " %u", 0);
            } if contains_upper_u == true {
                post_command = split_string(&exec_command, " %U", 1);
                exec_command = split_string(&exec_command, " %U", 0);
            }
            let command = format!("{} {} {}", exec_command, link, post_command);

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
                    .expect("Failed to open link with desired browser");
                quit::with_code(1);
            });
            button_container.append(&button);
            button_container.append(&label);
            app_container.append(&button_container);
        }
    } else {
        let empty_label = gtk::Label::new(Some("There's no desktop files!"));
        app_container.append(&empty_label);
    };
    
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

// Config
fn get_app_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| { 
        println!("There's no home directory!");
        quit::with_code(0);
    });
    let apps_path = PathBuf::from(home).join(".config/linko");
    if ! apps_path.exists() {
        create_dir_all(&apps_path).unwrap_or_else(|e| {
            println!("Failed to create config directory: {}", e);
        });
    }
    apps_path
}

// Helper
fn split_string(line: &str, cut: &str, position: usize) -> String {
    let almost_cut = line.split_once(cut).unwrap_or_default();
    // Isn't necessary to handle the case where the string isn't cutted
    let cutted;
    match position {
        0 => cutted = almost_cut.0.to_string(),
        1 => cutted = almost_cut.1.to_string(),
        _ => panic!(),
    };
    cutted
}
