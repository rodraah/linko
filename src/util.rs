use std::{
    fs::{self, create_dir_all, DirEntry, File},
    io::Read,
    path::PathBuf,
};

use gtk::{gio, prelude::*};
use gtk4 as gtk;

use crate::desktop_entry::DesktopEntry;
use crate::entry;

pub fn split_string(line: &str, cut: &str, position: usize) -> String {
    if position > 1 {
        return String::new();
    }
    match line.split_once(cut) {
        None => String::new(),
        Some((before, after)) => {
            if position == 0 {
                before.to_string()
            } else {
                after.to_string()
            }
        }
    }
}

pub fn get_target_url() -> Option<String> {
    let joined = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    if joined.is_empty() {
        None
    } else {
        Some(joined)
    }
}

pub fn launch(entry: &DesktopEntry, url: &str) -> std::io::Result<std::process::Child> {
    std::process::Command::new(&entry.program)
        .args(&entry.args)
        .arg(url)
        .spawn()
}

pub fn get_app_path() -> PathBuf {
    let apps_path = match std::env::var("HOME") {
        Ok(home) => PathBuf::from(home).join(".config/linko"),
        Err(_) => std::env::temp_dir().join("linko"),
    };
    if !apps_path.exists() {
        if let Err(e) = create_dir_all(&apps_path) {
            eprintln!("Error: Failed to create config directory: {}", e);
        }
    }
    apps_path
}

pub fn add_entries_dialog(
    parent: &gtk::ApplicationWindow,
    app_container: &gtk::Box,
    target_url: Option<String>,
) {
    let dialog = gtk::FileChooserDialog::builder()
        .title("Choose your browser's desktop entry: ")
        .action(gtk::FileChooserAction::Open)
        .build();
    dialog.set_transient_for(Some(parent));
    dialog.set_modal(true);
    dialog.set_destroy_with_parent(true);
    let default_path = gio::File::for_path("/usr/share/applications");
    if let Err(e) = dialog.set_current_folder(Some(&default_path)) {
        eprintln!(
            "Error: Failed to read \"usr/share/applications\" folder: {}",
            e
        );
    }
    dialog.add_buttons(&[
        ("Open", gtk::ResponseType::Accept),
        ("Cancel", gtk::ResponseType::Cancel),
    ]);
    let container = app_container.clone();
    dialog.connect_response(move |dialog, response| {
        if response == gtk::ResponseType::Accept {
            if let Some(file) = dialog.file() {
                if let Some(path) = file.path() {
                    match path.file_name().and_then(|n| n.to_str()) {
                        Some(file_name) => {
                            let destination = get_app_path().join(file_name);
                            if let Err(e) = fs::copy(&path, &destination) {
                                eprintln!("Error: Failed to copy desktop entry: {}", e);
                            } else {
                                while let Some(child) = container.first_child() {
                                    container.remove(&child);
                                }
                                entry::entry(&container, target_url.clone());
                            }
                        }
                        None => {
                            eprintln!("Error: Invalid file name for desktop entry");
                        }
                    }
                } else {
                    eprintln!("Error: Could not resolve chosen file path");
                }
            } else {
                eprintln!("Error: No file chosen");
            }
        };
        dialog.destroy()
    });
    dialog.show();
}

/// Split an `Exec=` value into program + args, respecting single/double
/// quotes and stripping freedesktop field codes (`%u`, `%U`, `%f`, ...).
/// `%%` is collapsed to a literal `%`.
fn parse_exec_value(exec: &str) -> (String, Vec<String>) {
    // Tokenize respecting single/double quotes.
    let mut tokens: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut in_token = false;

    for c in exec.chars() {
        if in_single {
            if c == '\'' {
                in_single = false;
            } else {
                current.push(c);
            }
        } else if in_double {
            if c == '"' {
                in_double = false;
            } else {
                current.push(c);
            }
        } else if c == '\'' {
            in_single = true;
            in_token = true;
        } else if c == '"' {
            in_double = true;
            in_token = true;
        } else if c.is_whitespace() {
            if in_token {
                tokens.push(std::mem::take(&mut current));
                in_token = false;
            }
        } else {
            current.push(c);
            in_token = true;
        }
    }
    if in_token {
        tokens.push(current);
    }

    const FIELD_CODES: [&str; 13] = [
        "%u", "%U", "%f", "%F", "%d", "%D", "%n", "%N", "%i", "%c", "%k", "%v", "%m",
    ];

    let mut args: Vec<String> = Vec::new();
    let mut program = String::new();
    let mut first = true;
    for token in tokens {
        if token == "%%" {
            if first {
                program = "%".to_string();
                first = false;
            } else {
                args.push("%".to_string());
            }
            continue;
        }
        if FIELD_CODES.contains(&token.as_str()) {
            // Remembered (URL/file placeholder) but intentionally not embedded.
            continue;
        }
        // Collapse escaped percent, then strip any embedded field codes.
        let mut cleaned = token.replace("%%", "%");
        for code in FIELD_CODES {
            if cleaned.contains(code) {
                cleaned = cleaned.replace(code, "");
            }
        }
        if cleaned.is_empty() {
            continue;
        }
        if first {
            program = cleaned;
            first = false;
        } else {
            args.push(cleaned);
        }
    }

    (program, args)
}

pub fn parse_desktop_file(
    app_desktop_path: DirEntry,
) -> Option<crate::desktop_entry::DesktopEntry> {
    let path = app_desktop_path.path();
    let fallback_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Unknown".to_string());

    let mut desktop_file = File::open(&path).ok()?;
    let mut desktop_content = String::new();
    desktop_file.read_to_string(&mut desktop_content).ok()?;

    let mut app_display_name: Option<String> = None;
    let mut app_exec_command: Option<String> = None;
    let mut app_display_icon: Option<String> = None;

    for line in desktop_content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue;
        };
        if trimmed.starts_with("Name=") && app_display_name.is_none() {
            let value = split_string(trimmed, "Name=", 1);
            if !value.trim().is_empty() {
                app_display_name = Some(value);
            }
            continue;
        };
        if trimmed.starts_with("Exec=") && app_exec_command.is_none() {
            let value = split_string(trimmed, "Exec=", 1);
            if !value.trim().is_empty() {
                app_exec_command = Some(value);
            }
            continue;
        };
        if trimmed.starts_with("Icon=") && app_display_icon.is_none() {
            let value = split_string(trimmed, "Icon=", 1);
            if !value.trim().is_empty() {
                app_display_icon = Some(value);
            }
            continue;
        };
    }

    let exec_raw = app_exec_command?;
    let (program, args) = parse_exec_value(&exec_raw);
    if program.trim().is_empty() {
        return None;
    }

    Some(DesktopEntry {
        name: app_display_name.unwrap_or(fallback_name),
        icon: app_display_icon.unwrap_or_else(|| "web-browser".to_string()),
        program,
        args,
    })
}
