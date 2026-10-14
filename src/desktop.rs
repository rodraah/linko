use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::desktop_entry::{BrowserCandidate, DesktopEntry};

fn split_string(line: &str, cut: &str, position: usize) -> String {
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
pub fn scan_browser_candidates() -> Vec<BrowserCandidate> {
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
    let mut by_file: HashMap<String, RawCandidate> = HashMap::new();
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

pub fn parse_desktop_file(path: &Path) -> Option<DesktopEntry> {
    let fallback_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Unknown".to_string());

    let mut desktop_file = File::open(path).ok()?;
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

pub fn installed_entries() -> Vec<(String, DesktopEntry)> {
    let mut out = Vec::new();
    let dir_entries = match fs::read_dir(crate::launcher::get_app_path()) {
        Ok(entries) => entries.collect::<Vec<_>>(),
        Err(e) => {
            eprintln!("Error: Failed to read application directory: {}", e);
            return Vec::new();
        }
    };
    for app_desktop_path in dir_entries {
        let app_desktop_path = match app_desktop_path {
            Ok(entry) => entry,
            Err(e) => {
                eprintln!("Error: Failed to open an app path: {}", e);
                continue;
            }
        };
        let app_path_string = app_desktop_path.file_name().to_string_lossy().into_owned();
        if !app_path_string.ends_with(".desktop") {
            println!("\n{} is not a desktop entry, skipping it.", app_path_string);
            continue;
        }
        let app = match parse_desktop_file(&app_desktop_path.path()) {
            Some(app) => app,
            None => {
                eprintln!(
                    "Warning: Skipping unreadable desktop entry: {}",
                    app_path_string
                );
                continue;
            }
        };
        out.push((app_path_string, app));
    }
    out
}

pub fn install_candidate(source: &Path, file_name: &str) -> std::io::Result<u64> {
    let destination = crate::launcher::get_app_path().join(file_name);
    fs::copy(source, &destination)
}

pub fn remove_entry(file_name: &str) -> std::io::Result<()> {
    let path = crate::launcher::get_app_path().join(file_name);
    fs::remove_file(&path)
}
