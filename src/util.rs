use std::{
    cell::Cell,
    fs::{self, create_dir_all, DirEntry, File},
    io::Read,
    path::{Path, PathBuf},
    rc::Rc,
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

struct BrowserCandidate {
    name: String,
    icon: String,
    file_name: String,
    source_path: PathBuf,
}

struct RawCandidate {
    candidate: BrowserCandidate,
    exec: String,
    mime_type: String,
    categories: String,
}

/// Lightweight parse of a `.desktop` file for the picker list.
/// Returns `None` when the file cannot be read or has an empty `Exec=`.
fn parse_candidate_file(path: &Path) -> Option<RawCandidate> {
    let file_name = path.file_name().map(|n| n.to_string_lossy().into_owned())?;
    if !file_name.ends_with(".desktop") {
        return None;
    }

    let mut file = File::open(path).ok()?;
    let mut content = String::new();
    file.read_to_string(&mut content).ok()?;

    let mut name: Option<String> = None;
    let mut exec: Option<String> = None;
    let mut icon: Option<String> = None;
    let mut mime_type: Option<String> = None;
    let mut categories: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with("Name=") && name.is_none() {
            let value = split_string(trimmed, "Name=", 1);
            if !value.trim().is_empty() {
                name = Some(value);
            }
        } else if trimmed.starts_with("Exec=") && exec.is_none() {
            let value = split_string(trimmed, "Exec=", 1);
            if !value.trim().is_empty() {
                exec = Some(value);
            }
        } else if trimmed.starts_with("Icon=") && icon.is_none() {
            let value = split_string(trimmed, "Icon=", 1);
            if !value.trim().is_empty() {
                icon = Some(value);
            }
        } else if trimmed.starts_with("MimeType=") && mime_type.is_none() {
            mime_type = Some(split_string(trimmed, "MimeType=", 1));
        } else if trimmed.starts_with("Categories=") && categories.is_none() {
            categories = Some(split_string(trimmed, "Categories=", 1));
        }
    }

    let exec_value = exec?;
    if exec_value.trim().is_empty() {
        return None;
    }

    Some(RawCandidate {
        candidate: BrowserCandidate {
            name: name.unwrap_or_else(|| file_name.clone()),
            icon: icon.unwrap_or_else(|| "web-browser".to_string()),
            file_name,
            source_path: path.to_path_buf(),
        },
        exec: exec_value,
        mime_type: mime_type.unwrap_or_default(),
        categories: categories.unwrap_or_default(),
    })
}

fn is_browser_candidate(exec: &str, mime_type: &str, categories: &str) -> bool {
    let mime_lower = mime_type.to_lowercase();
    if mime_lower.contains("x-scheme-handler/http")
        || mime_lower.contains("x-scheme-handler/https")
        || mime_lower.contains("text/html")
    {
        return true;
    }
    if categories.to_lowercase().contains("webbrowser") {
        return true;
    }
    const KNOWN_BROWSERS: [&str; 8] = [
        "firefox", "chrome", "chromium", "brave", "edge", "opera", "vivaldi", "zen",
    ];
    let exec_lower = exec.to_lowercase();
    KNOWN_BROWSERS
        .iter()
        .any(|browser| exec_lower.contains(browser))
}

/// Scan system + user application dirs for `.desktop` files, keeping only
/// likely browsers. Falls back to the unfiltered list when the filter
/// matches nothing. Never panics on missing dirs.
///
/// Scanned dirs (in order; missing/unreadable ones are skipped):
/// system first (`/usr/share/applications`,
/// `/var/lib/snapd/desktop/applications`,
/// `/var/lib/flatpak/exports/share/applications`, each `$XDG_DATA_DIRS`
/// entry + `/applications`), then user dirs
/// (`~/.local/share/applications`,
/// `~/.local/share/flatpak/exports/share/applications`,
/// `$XDG_DATA_HOME` + `/applications` when defined).
/// Dedupe is by file name with last-wins, so user dirs override system
/// entries with the same file name.
fn scan_browser_candidates() -> Vec<BrowserCandidate> {
    fn push_unique(dirs: &mut Vec<PathBuf>, dir: PathBuf) {
        if !dirs.contains(&dir) {
            dirs.push(dir);
        }
    }

    let mut dirs: Vec<PathBuf> = Vec::new();
    push_unique(&mut dirs, PathBuf::from("/usr/share/applications"));
    push_unique(
        &mut dirs,
        PathBuf::from("/var/lib/snapd/desktop/applications"),
    );
    push_unique(
        &mut dirs,
        PathBuf::from("/var/lib/flatpak/exports/share/applications"),
    );
    if let Ok(data_dirs) = std::env::var("XDG_DATA_DIRS") {
        for part in data_dirs.split(':') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            push_unique(&mut dirs, PathBuf::from(part).join("applications"));
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        let home = PathBuf::from(home);
        push_unique(&mut dirs, home.join(".local/share/applications"));
        push_unique(
            &mut dirs,
            home.join(".local/share/flatpak/exports/share/applications"),
        );
    }
    if let Ok(data_home) = std::env::var("XDG_DATA_HOME") {
        let data_home = data_home.trim();
        if !data_home.is_empty() {
            push_unique(&mut dirs, PathBuf::from(data_home).join("applications"));
        }
    }

    // Dedupe by file name so the same entry shown once (user dir wins).
    let mut by_file: std::collections::HashMap<String, RawCandidate> =
        std::collections::HashMap::new();
    for dir in &dirs {
        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for dir_entry in entries {
            let dir_entry = match dir_entry {
                Ok(entry) => entry,
                Err(_) => continue,
            };
            if let Some(raw) = parse_candidate_file(&dir_entry.path()) {
                by_file.insert(raw.candidate.file_name.clone(), raw);
            }
        }
    }

    if by_file.is_empty() {
        return Vec::new();
    }

    let mut filtered: Vec<BrowserCandidate> = Vec::new();
    let mut all: Vec<BrowserCandidate> = Vec::new();
    for (_, raw) in by_file {
        let browser_like = is_browser_candidate(&raw.exec, &raw.mime_type, &raw.categories);
        if browser_like {
            filtered.push(raw.candidate);
        } else {
            all.push(raw.candidate);
        }
    }

    let mut candidates = if filtered.is_empty() {
        // Filter matched nothing: show everything so the dialog never blocks.
        filtered.extend(all);
        filtered
    } else {
        // Filter matched something; still include the rest? No — keep only
        // browsers (fallback 1). Non-matching entries stay hidden.
        filtered
    };
    candidates.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    candidates
}

fn show_legacy_file_chooser(
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
    let filter = gtk::FileFilter::new();
    filter.set_name(Some("Desktop entries"));
    filter.add_pattern("*.desktop");
    dialog.add_filter(&filter);
    dialog.set_filter(&filter);
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

pub fn add_entries_dialog(
    parent: &gtk::ApplicationWindow,
    app_container: &gtk::Box,
    target_url: Option<String>,
) {
    let candidates = scan_browser_candidates();
    if candidates.is_empty() {
        show_legacy_file_chooser(parent, app_container, target_url);
        return;
    }

    let dialog = gtk::Dialog::builder()
        .title("Add browsers")
        .modal(true)
        .build();
    dialog.set_transient_for(Some(parent));
    dialog.set_destroy_with_parent(true);
    dialog.set_default_width(400);
    dialog.set_default_height(400);

    let content = dialog.content_area();
    content.set_spacing(6);

    let scrolled = gtk::ScrolledWindow::new();
    scrolled.set_vexpand(true);
    scrolled.set_hexpand(true);

    let list = gtk::ListBox::new();
    list.set_selection_mode(gtk::SelectionMode::None);
    scrolled.set_child(Some(&list));
    content.append(&scrolled);

    let added_any = Rc::new(Cell::new(false));

    for candidate in candidates {
        let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        row_box.set_margin_top(6);
        row_box.set_margin_bottom(6);
        row_box.set_margin_start(12);
        row_box.set_margin_end(12);

        let image = gtk::Image::builder()
            .icon_name(&candidate.icon)
            .pixel_size(24)
            .build();

        let text_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
        text_box.set_hexpand(true);

        let name_label = gtk::Label::new(Some(&candidate.name));
        name_label.set_xalign(0.0);
        name_label.set_single_line_mode(true);

        let file_label = gtk::Label::builder()
            .label(&candidate.file_name)
            .css_classes(vec!["dim-label", "caption"])
            .build();
        file_label.set_xalign(0.0);
        file_label.set_single_line_mode(true);

        text_box.append(&name_label);
        text_box.append(&file_label);

        let add_button = gtk::Button::with_label("Add");
        let source_path = candidate.source_path.clone();
        let file_name = candidate.file_name.clone();
        let added_flag = added_any.clone();
        add_button.connect_clicked(move |button| {
            let destination = get_app_path().join(&file_name);
            match fs::copy(&source_path, &destination) {
                Ok(_) => {
                    println!("Added desktop entry: {}", file_name);
                    button.set_label("Added");
                    button.set_sensitive(false);
                    added_flag.set(true);
                }
                Err(e) => {
                    eprintln!("Error: Failed to copy desktop entry: {}", e);
                }
            }
        });

        row_box.append(&image);
        row_box.append(&text_box);
        row_box.append(&add_button);

        let row = gtk::ListBoxRow::new();
        row.set_child(Some(&row_box));
        list.append(&row);
    }

    dialog.add_buttons(&[("Done", gtk::ResponseType::Close)]);

    let container = app_container.clone();
    let added_flag = added_any.clone();
    dialog.connect_response(move |dialog, _| {
        dialog.destroy();
        // Single reload on close, only when something was actually added.
        if added_flag.get() {
            while let Some(child) = container.first_child() {
                container.remove(&child);
            }
            entry::entry(&container, target_url.clone());
        }
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
